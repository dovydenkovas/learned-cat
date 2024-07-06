use crate::model::Settings;
use clap::arg;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

mod network;
use network::Request;
mod presenter;
use presenter::Presenter;
mod model;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = get_arguments();

    match arguments.subcommand() {
        Some(("init", _)) => {
            let path = crate::model::get_daemon_dir_path();
            let path = Path::new(&path);
            model::init::init_server(path)
        },
        Some(("run", _)) => {
            let settings = model::read_settings()?;
            start_server(settings)?
        },
        Some(("export-results", args)) => {
            let settings = model::read_settings()?;
            export_results(
            settings.result_path,
            args.get_one::<String>("filename")
                .or(Some(&"output.csv".to_string()))
                .unwrap()
                .to_string(),
        )?
        },
        Some((&_, _)) => eprintln!("Неизвестная команда."),
        None => eprintln!("Необходимо указать команду. Для просмотра доступных кооманд используйте переметр --help"),
    };
    Ok(())
}

fn start_server(settings: Settings) -> Result<(), Box<dyn Error>> {
    let mut presenter: Presenter = Presenter::new(settings);
    server_mainloop(&mut presenter);
    Ok(())
}

fn export_results(results_filename: String, output_filename: String) -> Result<(), Box<dyn Error>> {
    let result_path = model::get_daemon_dir_path() + "/" + results_filename.as_str();
    let results = model::load_results(&result_path);
    let mut file = match std::fs::File::create(output_filename.clone()) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Не могу создать файл {output_filename}: {err}");
            std::process::exit(1);
        }
    };

    for vars in results.values() {
        for var in &vars.variants {
            if var.result.is_some() {
                let out = format!(
                    "{},{},{},{},{}\n",
                    var.testname,
                    var.username,
                    var.timestamp.unwrap().date_naive(),
                    var.timestamp.unwrap().time().format("%H:%M:%S"),
                    var.result.unwrap()
                );

                println!("{}", out);
                let _ = file.write(out.as_bytes());
            }
        }
    }
    Ok(())
}

fn get_arguments() -> clap::ArgMatches {
    clap::Command::new("learned-cat-server")
        .version("0.1.0")
        .author("Aleksandr Dovydenkov. <asdovydenkov@gmail.com>")
        .about("Сервер тестирования в терминале. ")
        .subcommand(
            clap::Command::new("init")
                .short_flag('i')
                .about("создать файлы сервера в каталоге /opt/learned-cat")

                .arg(arg!([postgres_credentials])),
        )

        .subcommand(
            clap::Command::new("run")
                .short_flag('r')
                .about("запустить сервер")

        )

        .subcommand(
            clap::Command::new("export-results")
                .short_flag('o')
                .about("экспортировать результаты тестирования в виде csv таблицы следующего формата: <test>,<student>,<date>,<time>,<result>")
                .arg(arg!([filename]).required(true)),
        )

        .get_matches()
}

/// Open listener and run mainloop
fn server_mainloop(presenter: &mut Presenter) {
    let address = presenter.model.lock().unwrap().get_server_address();
    println!("* Открываю порт сервера: {}", address);
    let listener = TcpListener::bind(address).expect("Не могу открыть соединение");

    println!("* Запускаю главный цикл\n");
    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => match handle_client(&mut stream, presenter) {
                    Ok(()) => (),
                    Err(err) => eprintln!("{err:?}"),
                },
                Err(err) => eprintln!("{err:?}"),
            }
        }
    }
}

fn handle_client(stream: &mut TcpStream, presenter: &mut Presenter) -> Result<(), Box<dyn Error>> {
    let mut request = [0 as u8; 5000];
    let n_bytes = stream.read(&mut request)?;
    let request = bincode::deserialize::<Request>(&request[0..n_bytes])?;

    print!("[{}] {:?} -> ", chrono::Utc::now(), request);
    let response = presenter.serve_connection(request);
    println!("{:?}", response);
    let response = bincode::serialize(&response)?;

    stream.write(&response)?;
    Ok(())
}
