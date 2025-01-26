use clap::arg;
use learned_cat::server::SocketServer;
use learned_cat::Controller;
use learned_cat_database::TestDatabase;
use std::env::set_current_dir;
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use learned_cat_config::TomlConfig;
use learned_cat_interfaces::{Config, Statistic};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = arguments();
    let root_path = daemon_dir_path();

    match arguments.subcommand() {
        Some(("run", _)) => {
            start_server(root_path)?
        },
        Some(("export-results", args)) => {
            let output_filename = args.get_one::<String>("filename")
                             .or(Some(&"output.csv".to_string()))
                             .unwrap()
                             .to_string();
            export_results(root_path, output_filename)?
        },
        Some((&_, _)) => eprintln!("Неизвестная команда."),
        None => eprintln!("Необходимо указать команду. Для просмотра доступных команд используйте переметр --help"),
    };
    Ok(())
}

/// Запуск сервера.
fn start_server(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let config = TomlConfig::new(&path)?;

    let tests_path = Path::new(&path).join(&config.settings().result_path.clone());
    let database = TestDatabase::new(tests_path.to_str().unwrap().to_string());
    let server = SocketServer::new(config.settings().server_address.clone());

    set_daemon_dir(&path).expect("Невозможно перейти в директорию с файлами сервера.");

    let mut controller = Controller::new(
        Box::new(config),
        Box::new(database),
        Arc::new(Mutex::new(server)),
    );
    controller.run();
    Ok(())
}

/// Сохранение результатов тестирования в файл.
fn export_results(root_path: PathBuf, output_filename: String) -> Result<(), Box<dyn Error>> {
    // Load Statistic
    let config = TomlConfig::new(&root_path)?;
    let tests_path = Path::new(&root_path).join(&config.settings().result_path.clone());
    let mut statistic: Box<dyn Statistic> =
        Box::new(TestDatabase::new(tests_path.to_str().unwrap().to_string()));

    // Create output file
    let mut file = match std::fs::File::create(output_filename.clone()) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Не могу создать файл {output_filename}: {err}");
            std::process::exit(1);
        }
    };

    // Save output file
    for user in &statistic.users() {
        for result in &statistic.results(user) {
            let out = format!(
                "{},{},{},{},{}\n",
                result.testname,
                result.username,
                result.start_datetime.to_string(),
                result.end_datetime.to_string(),
                result.mark
            );

            println!("{}", out);
            let _ = file.write(out.as_bytes());
        }
    }

    Ok(())
}

/// Аргументы командной строки.
fn arguments() -> clap::ArgMatches {
    clap::Command::new("learned-cat-server")
        .version("1.0.0")
        .author("Aleksandr Dovydenkov. <asdovydenkov@gmail.com>")
        .about("Сервер тестирования в терминале. ")

        .subcommand(
            clap::Command::new("run")
                .short_flag('r')
                .about("запустить сервер")
        )

        .subcommand(
            clap::Command::new("export-results")
                .short_flag('o')
                .about("экспортировать результаты тестирования в виде csv таблицы следующего формата: <test>,<student>,<time_begin>,<time_end>,<result>")
                .arg(arg!([filename]).required(true)),
        )

        .get_matches()
}

/// Путь к файлам сервера
fn daemon_dir_path() -> PathBuf {
    match std::env::var("LEARNED_CAT_PATH") {
        Ok(v) => PathBuf::from(v),
        Err(_) => PathBuf::from("/opt/learned-cat"),
    }
}

/// Перейти в директорию с файлами сервера
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
