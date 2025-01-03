use examiner::{Answer, Question, TestSettings};

pub mod errors;
pub mod examiner;
pub mod init;
pub mod network;
pub mod parsetest;
pub mod presenter;

/// Интерфейс взаимодействия Сервера и Экзаменатора.
trait Server {
    /// Взять запрос из очереди запроса.
    fn pop_request(&mut self) -> network::Request;

    /// Отправить ответ на запрос.
    fn push_response(&mut self, response: network::Response);
}

/// Интерфейс взаимодействия Экзаменатора с Базой данных.
trait Database {
    /// Проверить валидность пользователя username.
    fn has_user(username: String) -> bool;

    /// Проверить валидность теста testname.
    fn has_test(testname: String) -> bool;

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(username: String, testname: String) -> bool;

    /// Сколько попыток для прохождения теста testname осталось у пользователя username.
    fn remaining_attempts_number(username: String, testname: String) -> u32;

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(username: String) -> Vec<String>;

    /// Получить параметры теста testname.
    fn test_settings(testname: String) -> TestSettings;

    /// Получить текст вопроса question_id теста testname.
    fn question(testname: String, question_id: u64) -> Question;

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(testname: String, question_id: u64) -> Answer;

    /// Получить баллы за тест testname для пользователя username.
    fn mark(username: String, testname: String) -> Vec<f32>;

    /// Сохранить баллы за тест testname для пользователя username.
    fn add_mark(username: String, testname: String, mark: f32);
}
