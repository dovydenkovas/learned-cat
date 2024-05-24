use std::env::set_current_dir;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use clap::arg;

mod network;
use network::Request;
mod presenter;
use presenter::Presenter;
mod model;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = get_arguments();
    set_daemon_dir()?;

    match arguments.subcommand() {
        Some(("init", _)) => model::init::init_server(),
        Some(("start", _)) => start_server()?,
        Some(("export-results", args)) => export_results(
            args.get_one::<String>("filename")
                .or(Some(&"output.csv".to_string()))
                .unwrap()
                .to_string(),
        )?,
        Some((&_, _)) => eprintln!("Неизвестная команда."),
        None => eprintln!("Неизвестная команда."),
    };
    Ok(())
}

fn start_server() -> Result<(), Box<dyn Error>> {
    let mut presenter: Presenter = Presenter::new();
    server_mainloop(&mut presenter);
    Ok(())
}

fn export_results(filename: String) -> Result<(), Box<dyn Error>> {
    //presenter.export_results(filename); // arguments.get_one::<String>("filename").unwrap().to_string());
    todo!();
}

fn set_daemon_dir() -> Result<(), Box<dyn Error>> {
    let root = std::path::Path::new("/home/asd/code/desktop/sshtest/sshtest-dir"); // TODO
    if set_current_dir(&root).is_err() {
        eprintln!(
            "Ошибка доступа к каталогу сервера {}.",
            root.to_str().unwrap()
        );
        eprintln!("Проверьте, что каталог существует, и у процесса есть у нему доступ.");
        return Err(Box::new(std::fmt::Error));
    }
    Ok(())
}

fn get_arguments() -> clap::ArgMatches {
    clap::Command::new("sshtest-server")
        .version("0.1.0")
        .author("Aleksandr Dovydenkov. <asdovydenkov@gmail.com>")
        .about("Сервер тестирования в терминале. ")
        .subcommand(
            clap::Command::new("init")
                .about("создать файлы сервера в каталоге /opt/sshtest")
                .arg(arg!([postgres_credentials])),
        )

        .subcommand(
            clap::Command::new("start")
                .about("запустить сервер")
        )

        .subcommand(
            clap::Command::new("export-results")

            .about("экспортировать результаты тестирования в виде csv таблицы следующего формата: <student>;<test>;<date>;<result>")
                .arg(arg!([filename]).required(true)),
        )

        .get_matches()
}

/// Open listener and run mainloop
fn server_mainloop(presenter: &mut Presenter) {
    let address = presenter.model.get_server_address();
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
