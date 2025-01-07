use schema::{Answer, Question};

pub mod network;
pub mod schema;
pub mod settings;

/// Интерфейс взаимодействия Сервера и Экзаменатора.
pub trait Server {
    /// Взять запрос из очереди запроса.
    fn pop_request(&mut self) -> Option<network::Request>;

    /// Отправить ответ на запрос.
    fn push_response(&mut self, response: network::Response);
}

/// Интерфейс взаимодействия Экзаменатора с настройками.
pub trait Config {
    /// Существует ли пользователь?
    fn has_user(&mut self, username: &String) -> bool;

    /// Проверить валидность теста testname.
    fn has_test(&mut self, testname: &String) -> bool;

    /// Получить параметры теста testname.
    fn test_settings(&mut self, testname: &String) -> settings::TestSettings;

    /// Получить описание теста.
    fn test_banner(&mut self, testname: &String) -> String;

    /// Получить текст вопроса question_id теста testname.
    fn question(&mut self, testname: &String, question_id: usize) -> Question;

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&mut self, testname: &String, question_id: usize) -> Answer;

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&mut self, username: &String, testname: &String) -> bool;

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&mut self, username: &String) -> Vec<String>;

    /// Получить параметры сервера.
    fn settings(&mut self, testname: &String) -> settings::Settings;
}

/// Интерфейс взаимодействия Экзаменатора с Базой данных.
pub trait Database {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String>;

    /// Сколько попыток для прохождения теста testname потратил пользователь username.
    fn attempts_counter(&mut self, username: &String, testname: &String) -> u32;

    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32>;

    /// Сохранить баллы за тест testname для пользователя username.
    fn append_mark(&mut self, username: &String, testname: &String, mark: f32);
}
