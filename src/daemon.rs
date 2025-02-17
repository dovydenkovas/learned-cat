use clap::arg;
use lc_database::TestDatabase;
use lc_examiner::examiner::Examiner;
use lc_exammanager::exammanager::ExamManager;
use lc_reporter::Reporter;
use lc_server::socketserver::SocketServer;
use log4rs::append::{console::ConsoleAppender, file::FileAppender};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env::set_current_dir;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use log::{debug, error};

use lc_config::TomlConfig;
use lc_examiner::Config;
use lc_reporter::Statistic;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = arguments();
    let root_path = daemon_dir_path();

    match arguments.subcommand() {
        Some(("run", _)) => {
            start_server(root_path)?
        },
        Some(("export-results", args)) => {
            let output_filename = PathBuf::from(args.get_one::<String>("filename")
                             .or(Some(&"output.csv".to_string()))
                             .unwrap());
            export_results(root_path, output_filename)?
        },
        Some((&_, _)) => error!("Неизвестная команда."),
        None => error!("Необходимо указать команду. Для просмотра доступных команд используйте переметр --help"),
    };
    Ok(())
}

/// Сохранить результаты тестирования в файл
fn export_results(root_path: PathBuf, output_filename: PathBuf) -> Result<(), Box<dyn Error>> {
    // Подключаемся к настройкам и базе данных
    let config = TomlConfig::new(&root_path).unwrap();
    let tests_path = Path::new(&root_path).join(&config.settings().result_path.clone());
    let statistic: Box<dyn Statistic> =
        Box::new(TestDatabase::new(tests_path.to_str().unwrap().to_string()));

    // Запускаем генератор отчетов
    let mut reporter: Box<dyn Reporter> =
        Box::new(lc_reporter::csv_reporter::CsvReporter::new(statistic));

    reporter.save_report(output_filename);

    Ok(())
}

/// Запуск сервера.
fn start_server(path: PathBuf) -> Result<(), Box<dyn Error>> {
    set_daemon_dir(&path).expect("Невозможно перейти в директорию с файлами сервера.");
    let config = TomlConfig::new(&path)?;

    start_logger(config.settings().log_level.clone());

    debug!("Открываю базу данных.");
    let tests_path = Path::new(&path).join(&config.settings().result_path.clone());
    let database = TestDatabase::new(tests_path.to_str().unwrap().to_string());

    debug!("Запуска сервер.");
    let server = SocketServer::new(config.settings().server_address.clone());

    debug!("Подготавливаю правила обработки тестов.");
    let examiner = Examiner::new(Box::new(config), Box::new(database));

    debug!("Подготовка всех систем.");
    let mut controller = ExamManager::new(examiner, Arc::new(Mutex::new(server)));

    debug!("Запуск.");
    controller.run();
    Ok(())
}

fn str2log_level(log_level: String) -> log::LevelFilter {
    if log_level.as_str() == "debug" {
        log::LevelFilter::Debug
    } else if log_level.as_str() == "info" {
        log::LevelFilter::Info
    } else if log_level.as_str() == "warn" {
        log::LevelFilter::Warn
    } else if log_level.as_str() == "error" {
        log::LevelFilter::Error
    } else {
        eprintln!("Уровень логирования установлен как debug");
        log::LevelFilter::Debug
    }
}

/// Настройка и запуск логирования
fn start_logger(log_level: String) {
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
                .build(str2log_level(log_level)),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
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
