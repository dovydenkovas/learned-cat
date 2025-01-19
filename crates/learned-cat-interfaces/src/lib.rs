use schema::{Answer, Question, TestRecord};

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
    fn has_user(&self, username: &String) -> bool;

    /// Проверить валидность теста testname.
    fn has_test(&self, testname: &String) -> bool;

    /// Получить параметры теста testname.
    fn test_settings(&self, testname: &String) -> settings::TestSettings;

    /// Получить описание теста.
    fn test_banner(&self, testname: &String) -> String;

    /// Получить текст вопроса question_id теста testname.
    fn question(&self, testname: &String, question_id: usize) -> Question;

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&self, testname: &String, question_id: usize) -> Answer;

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&self, username: &String, testname: &String) -> bool;

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&self, username: &String) -> Vec<String>;

    /// Получить параметры сервера.
    fn settings(&self) -> settings::Settings;
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
    );
}

pub trait Statistic {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String>;

    /// Список результатов конкретного пользователя.
    fn results(&mut self, username: &String) -> TestRecord;
}
