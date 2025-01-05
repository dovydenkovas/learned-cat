use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::{collections::HashMap, path::PathBuf};

// use self::schema::Answer::dsl::*;
// use self::schema::Question::dsl::*;
// use self::schema::Test::dsl::*;
// use self::schema::Variant::dsl::*;
use self::schema::User::dsl::*;

pub mod models;
pub mod schema;

use learned_cat_interfaces::{
    schema::{Answer, Question},
    settings::{Settings, TestSettings},
    Database,
};

pub struct TestDatabase {
    connection: SqliteConnection,
}

impl TestDatabase {
    pub fn new(_settings: &Settings, _tests_directory_path: PathBuf) -> TestDatabase {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        TestDatabase { connection }
    }
}

impl Database for TestDatabase {
    /// Сколько попыток для прохождения теста testname осталось у пользователя username.
    fn remaining_attempts_number(&mut self, username: &String, testname: &String) -> u32;

    // Функции управления пользователем
    /// Добавить пользователя.
    fn append_user(&mut self, username: &String);
    /// Удалить пользователя.
    fn remove_user(&mut self, username: &String);
    /// Существует ли пользователь?
    fn has_user(&mut self, username: &String) -> bool {
        let res = User.filter(name.eq(username)).first(&mut self.connection);
        return res.is_ok();
    }

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
