#![allow(unused)]
use std::{collections::HashMap, path::PathBuf};

use learned_cat_interfaces::{
    schema::{Answer, Question, Variants},
    settings::{Settings, TestSettings},
    Database,
};

type TestResults = HashMap<String, Variants>;

pub struct TestDatabase {}

impl TestDatabase {
    pub fn new(settings: &Settings, tests_directory_path: PathBuf) -> TestDatabase {
        TestDatabase {}
    }
}

impl Database for TestDatabase {
    /// Проверить валидность пользователя username.
    fn has_user(&mut self, username: &String) -> bool {
        true
    }

    /// Проверить валидность теста testname.
    fn has_test(&mut self, testname: &String) -> bool {
        true
    }

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&mut self, username: &String, testname: &String) -> bool {
        true
    }

    /// Сколько попыток для прохождения теста testname осталось у пользователя username.
    fn remaining_attempts_number(&mut self, username: &String, testname: &String) -> u32 {
        3
    }

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&mut self, username: &String) -> Vec<(String, String)> {
        vec![]
    }

    /// Получить параметры теста testname.
    fn test_settings(&mut self, testname: &String) -> TestSettings {
        TestSettings::default()
    }

    /// Получить описание теста.
    fn test_banner(&mut self, testname: &String) -> String {
        "Описание".to_string()
    }

    /// Получить текст вопроса question_id теста testname.
    fn question(&mut self, testname: &String, question_id: u64) -> Question {
        Question {
            question: "2+2?".to_string(),
            answers: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
            ],
            correct_answers: vec![3],
        }
    }

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&mut self, testname: &String, question_id: u64) -> Answer {
        Answer { answers: vec![3] }
    }

    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32> {
        vec![]
    }

    /// Сохранить баллы за тест testname для пользователя username.
    fn add_mark(&mut self, username: &String, testname: &String, mark: f32) {
        println!("!!!");
    }
}
