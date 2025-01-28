use clap::arg;
use learned_cat::server::SocketServer;
use learned_cat::Controller;
use learned_cat_database::TestDatabase;
use log4rs::append::{console::ConsoleAppender, file::FileAppender};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env::set_current_dir;
use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use log::{debug, error};

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
        Some((&_, _)) => error!("Неизвестная команда."),
        None => error!("Необходимо указать команду. Для просмотра доступных команд используйте переметр --help"),
    };
    Ok(())
}

fn start_logger() {
    let logconsole = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)} {h({l})}]: {M} - {m}\n",
        )))
        .build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)} {l}]: {M} - {m}\n",
        )))
        .build("log/output.log")
        .unwrap();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder().build("logconsole", Box::new(logconsole)))
        .build(
            Root::builder()
                .appender("logconsole")
                .appender("logfile")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}

/// Запуск сервера.
fn start_server(path: PathBuf) -> Result<(), Box<dyn Error>> {
    start_logger();
    set_daemon_dir(&path).expect("Невозможно перейти в директорию с файлами сервера.");
    debug!("Считываю настройки.");
    let config = TomlConfig::new(&path)?;
    debug!("Открываю базу данных.");
    let tests_path = Path::new(&path).join(&config.settings().result_path.clone());
    let database = TestDatabase::new(tests_path.to_str().unwrap().to_string());
    debug!("Запуска сервер.");
    let server = SocketServer::new(config.settings().server_address.clone());

    debug!("Подготовка всех систем.");
    let mut controller = Controller::new(
        Box::new(config),
        Box::new(database),
        Arc::new(Mutex::new(server)),
    );

    debug!("Запуск.");
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
            error!("Не могу создать файл {output_filename}: {err}");
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

            print!("{}", out);
            let _ = file.write(out.as_bytes());
        }
        println!();
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
        error!(
            "Ошибка доступа к каталогу сервера {}.",
            path.as_ref().to_str().unwrap()
        );
        error!("Проверьте, что каталог существует, и у процесса есть у нему доступ.");
        return Err(Box::new(std::fmt::Error));
    }
    Ok(())
}
