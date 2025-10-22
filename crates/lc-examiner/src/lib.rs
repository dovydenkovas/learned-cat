pub mod examiner;
pub mod network;
pub mod schema;
pub mod settings;

use std::path::PathBuf;

use schema::{Answer, Question, Variant};
use settings::{Settings, TestSettings};

pub mod pexpect;

/// Директории, содержащие файлы программы.
pub struct DaemonPaths {
    /// Директория хранения настроек и тестов, по умолчанию /etc/learned-cat.
    pub settings: PathBuf,
    /// Директория сохранения результатов, по умолчанию /var/lib/learned-cat.
    pub database: PathBuf,
}

impl DaemonPaths {
    pub fn new() -> DaemonPaths {
        let settings = match std::env::var("LEARNED_CAT_SETTINGS") {
            Ok(v) => PathBuf::from(v),
            Err(_) => PathBuf::from("/etc/learned-cat"),
        };

        let database = match std::env::var("LEARNED_CAT_DATABASE") {
            Ok(v) => PathBuf::from(v),
            Err(_) => PathBuf::from("/var/lib/learned-cat"),
        };

        DaemonPaths { settings, database }
    }
}

/// Интерфейс взаимодействия Экзаменатора с настройками.
pub trait Config {
    /// Существует ли пользователь?
    fn has_user(&self, username: &String) -> bool;

    /// Проверить валидность теста testname.
    fn has_test(&self, testname: &String) -> bool;

    /// Получить параметры теста testname.
    fn test_settings(&self, testname: &String) -> Option<TestSettings>;

    /// Получить описание теста.
    fn test_banner(&self, testname: &String) -> Option<String>;

    /// Получить текст вопроса question_id теста testname.
    fn question(&self, testname: &String, question_id: usize) -> Option<Question>;

    /// Получить количество вопросов в тесте.
    fn questions_count(&self, testname: &String) -> Option<usize>;

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&self, testname: &String, question_id: usize) -> Option<Answer>;

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&self, username: &String, testname: &String) -> bool;

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&self, username: &String) -> Vec<String>;

    /// Получить параметры сервера.
    fn settings(&self) -> Settings;
}

/// Интерфейс взаимодействия Экзаменатора с Базой данных.
pub trait Database {
    /// Сколько попыток для прохождения теста testname потратил пользователь username.
    fn attempts_counter(&mut self, username: &String, testname: &String) -> u32;

    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32>;

    /// Сохранить баллы за тест testname для пользователя username.
    fn append_mark(
        &mut self,
        username: &String,
        testname: &String,
        mark: f32,
        start_timestamp: &String,
        end_timestamp: &String,
        variant: &Variant,
    );
}
