pub mod database;
pub mod examiner;
pub mod init;
pub mod network;
pub mod parsetest;
pub mod schema;
pub mod server;
pub mod settings;

use schema::{Answer, Question};
use settings::TestSettings;

/// Интерфейс взаимодействия Сервера и Экзаменатора.
pub trait Server {
    /// Взять запрос из очереди запроса.
    fn pop_request(&mut self) -> Option<network::Request>;

    /// Отправить ответ на запрос.
    fn push_response(&mut self, response: network::Response);
}

/// Интерфейс взаимодействия Экзаменатора с Базой данных.
pub trait Database {
    /// Проверить валидность пользователя username.
    fn has_user(&self, username: &String) -> bool;

    /// Проверить валидность теста testname.
    fn has_test(&self, testname: &String) -> bool;

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&self, username: &String, testname: &String) -> bool;

    /// Сколько попыток для прохождения теста testname осталось у пользователя username.
    fn remaining_attempts_number(&self, username: &String, testname: &String) -> u32;

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&self, username: &String) -> Vec<(String, String)>;

    /// Получить параметры теста testname.
    fn test_settings(&self, testname: &String) -> TestSettings;

    /// Получить описание теста.
    fn test_banner(&self, testname: &String) -> String;

    /// Получить текст вопроса question_id теста testname.
    fn question(&self, testname: &String, question_id: u64) -> Question;

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&self, testname: &String, question_id: u64) -> Answer;

    /// Получить баллы за тест testname для пользователя username.
    fn mark(&self, username: &String, testname: &String) -> Vec<f32>;

    /// Сохранить баллы за тест testname для пользователя username.
    fn add_mark(&self, username: &String, testname: &String, mark: f32);
}
