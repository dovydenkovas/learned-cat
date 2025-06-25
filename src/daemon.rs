use clap::arg;
use lc_database::TestDatabase;
use lc_examiner::examiner::Examiner;
use lc_exammanager::exammanager::ExamManager;
use lc_reporter::Reporter;
use lc_server::socketserver::SocketServer;
use log4rs::append::{console::ConsoleAppender, file::FileAppender};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use log::{debug, error};

use lc_config::TomlConfig;
use lc_examiner::{Config, DaemonPaths};
use lc_reporter::Statistic;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = arguments();
    let paths = DaemonPaths::new();

    match arguments.subcommand() {
        Some(("run", _)) => {
            start_server(paths)?
        },
        Some(("export-marks", args)) => {
            let output_filename = PathBuf::from(args.get_one::<String>("filename")
                             .or(Some(&"output.csv".to_string()))
                             .unwrap());
            export_marks(paths, output_filename)?
        },
        Some(("export-variants", args)) => {
            let username = args.get_one::<String>("user").unwrap();
            let testname = args.get_one::<String>("test").unwrap();
            export_variants(paths, &username, &testname)?
        },
        Some((&_, _)) => error!("Неизвестная команда."),
        None => error!("Необходимо указать команду. Для просмотра доступных команд используйте переметр --help"),
    };
    Ok(())
}

/// Сохранить результаты тестирования в файл
fn export_marks(paths: DaemonPaths, output_filename: PathBuf) -> Result<(), Box<dyn Error>> {
    // Подключаемся к настройкам и базе данных
    let config = TomlConfig::new(&paths).unwrap();
    let tests_path = Path::new(&paths.database).join(&config.settings().result_path.clone());
    let statistic: Box<dyn Statistic> =
        Box::new(TestDatabase::new(tests_path.to_str().unwrap().to_string()));

    // Запускаем генератор отчетов
    let mut reporter: Box<dyn Reporter> =
        Box::new(lc_reporter::csv_reporter::CsvReporter::new(statistic));

    reporter.marks_report(output_filename);

    Ok(())
}

/// Сохранить результаты тестирования в файл
fn export_variants(
    lc_paths: DaemonPaths,
    username: &String,
    testname: &String,
) -> Result<(), Box<dyn Error>> {
    // Подключаемся к настройкам и базе данных
    let config = TomlConfig::new(&lc_paths).unwrap();
    let tests_path = Path::new(&lc_paths.database).join(&config.settings().result_path.clone());
    let statistic: Box<dyn Statistic> =
        Box::new(TestDatabase::new(tests_path.to_str().unwrap().to_string()));

    // Запускаем генератор отчетов
    let mut reporter: Box<dyn Reporter> =
        Box::new(lc_reporter::csv_reporter::CsvReporter::new(statistic));

    reporter.variants_report(username, testname);

    Ok(())
}

/// Запуск сервера.
fn start_server(lc_paths: DaemonPaths) -> Result<(), Box<dyn Error>> {
    let config = TomlConfig::new(&lc_paths)?;

    start_logger(&lc_paths, config.settings().log_level.clone());

    debug!("Открываю базу данных.");
    let tests_path = Path::new(&lc_paths.database).join(&config.settings().result_path.clone());
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
fn start_logger(lc_paths: &DaemonPaths, log_level: String) {
    let logconsole = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{l}]: {M} - {m}\n")))
        .build();

    let filename = lc_paths.database.join("output.log");
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)} {l}]: {M} - {m}\n",
        )))
        .build(filename)
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
        .version("0.2.0")
        .author("Aleksandr Dovydenkov <asdovydenkov@yandex.ru>")
        .about("Сервер тестирования в терминале. ")

        .subcommand(
            clap::Command::new("run")
                .short_flag('r')
                .about("запустить сервер")
        )

        .subcommand(
            clap::Command::new("export-marks")
                .short_flag('m')
                .about("экспортировать результаты тестирования в виде csv таблицы следующего формата: <test>,<student>,<time_begin>,<time_end>,<result>")
                .arg(arg!([filename]).required(true)),
        )

        .subcommand(
                    clap::Command::new("export-variants")
                        .short_flag('v')
                        .about("экспортировать варианты тестов пользователя")
                        .arg(arg!([user]).required(true))
                        .arg(arg!([test]).required(true))
                )
        .get_matches()
}
