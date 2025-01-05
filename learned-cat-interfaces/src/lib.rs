use schema::{Answer, Question};
use settings::TestSettings;

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

/// Интерфейс взаимодействия Экзаменатора с Базой данных.
pub trait Database {
    /// Сколько попыток для прохождения теста testname осталось у пользователя username.
    fn remaining_attempts_number(&mut self, username: &String, testname: &String) -> u32;

    // Функции управления пользователем
    /// Добавить пользователя.
    fn append_user(&mut self, username: &String);
    /// Удалить пользователя.
    fn remove_user(&mut self, username: &String);
    /// Существует ли пользователь?
    fn has_user(&mut self, username: &String) -> bool;

    // Функции управления тестами.
    /// Добавить тест.
    fn append_test(&mut self, settings: &TestSettings);
    /// Удалить тест.
    fn remove_test(&mut self, testname: &String);
    /// Получить параметры теста testname.
    fn test_settings(&mut self, testname: &String) -> settings::TestSettings;
    /// Получить описание теста.
    fn test_banner(&mut self, testname: &String) -> String;
    /// Проверить валидность теста testname.
    fn has_test(&mut self, testname: &String) -> bool;

    // Функции управления вопросами.
    /// Добавить вопрос к тесту.
    fn append_question(&mut self, testname: &String, question: Question);
    /// Удалить вопрос из теста.
    fn remove_question(&mut self, testname: &String, question_id: u64);
    /// Получить текст вопроса question_id теста testname.
    fn question(&mut self, testname: &String, question_id: u64) -> Question;
    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&mut self, testname: &String, question_id: u64) -> Answer;
    /// Установить ответы на вопрос.
    fn set_answer(&mut self, testname: &String, question_id: u64, answer: Answer);

    // Функции управления вариантами.
    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&mut self, username: &String, testname: &String) -> bool;
    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&mut self, username: &String) -> Vec<(String, String)>;
    /// Разрешить пользователю проходить тест.
    fn append_access(&mut self, username: &String, testname: &String);
    /// Запретить пользователю проходить тест.
    fn remove_access(&mut self, username: &String, testname: &String);
    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32>;
    /// Сохранить баллы за тест testname для пользователя username.
    fn append_mark(&mut self, username: &String, testname: &String, mark: f32);
    /// Удалить результаты тестирования для пользователя.
    fn clear_marks(&mut self, username: &String, testname: &String);
}
