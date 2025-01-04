#![allow(unused)]
use clap::arg;
use learned_cat::examiner::Examiner;
use std::env::set_current_dir;
use std::error::Error;
use std::path::{Path, PathBuf};

use learned_cat::init;
use learned_cat::settings::{read_settings, Settings};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = get_arguments();

    let root_path = get_daemon_dir_path();
    match arguments.subcommand() {
        Some(("init", _)) => {
            init::init_server(&root_path)
        },
        Some(("run", _)) => {
            let settings = read_settings(&root_path)?;
            start_server(settings, root_path)?
        },
        Some(("export-results", args)) => {
            // TODO!
            // let settings = read_settings(&root_path)?;
            // export_results(
            // settings.result_path,
            // args.get_one::<String>("filename")
            //     .or(Some(&"output.csv".to_string()))
            //     .unwrap()
            //     .to_string(),
            // &root_path,
            // )?
        },
        Some((&_, _)) => eprintln!("Неизвестная команда."),
        None => eprintln!("Необходимо указать команду. Для просмотра доступных кооманд используйте переметр --help"),
    };
    Ok(())
}

fn start_server(settings: Settings, path: PathBuf) -> Result<(), Box<dyn Error>> {
    let tests_path = Path::new(&path).join(&settings.tests_directory_path);

    let database = learned_cat::database::TestDatabase::new(&settings, tests_path);
    let server = learned_cat::server::SocketServer::new(settings.server_address.clone());
    set_daemon_dir(&path).expect("Error init and start server");
    let _ = Examiner::new(Box::new(database), Box::new(server));
    Ok(())
}

fn export_results<P: AsRef<Path>>(
    results_filename: String,
    output_filename: String,
    root_path: P,
) -> Result<(), Box<dyn Error>> {
    /*
    let result_path = root_path.as_ref().join(results_filename);
    let results = load_results(&result_path);
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
    }*/
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

fn get_daemon_dir_path() -> PathBuf {
    match std::env::var("LEARNED_CAT_PATH") {
        Ok(v) => PathBuf::from(v),
        Err(_) => PathBuf::from("/opt/learned-cat"),
    }
}

fn set_daemon_dir<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    if set_current_dir(path.as_ref()).is_err() {
        eprintln!(
            "Ошибка доступа к каталогу сервера {}.",
            path.as_ref().to_str().unwrap()
        );
        eprintln!("Проверьте, что каталог существует, и у процесса есть у нему доступ.");
        return Err(Box::new(std::fmt::Error));
    }
    Ok(())
}
