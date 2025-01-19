#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
};

use std::error::Error;

use toml::from_str;

use learned_cat_interfaces::{
    self,
    schema::{Answer, Question},
    settings::{Settings, TestSettings},
    Config,
};

pub struct TomlConfig {
    settings: Settings,

    // Таблица пользователь: список доступных тестов.
    users: HashMap<String, HashSet<String>>,
    // Таблица тест: настройки.
    tests: HashMap<String, TestSettings>,
}

impl TomlConfig {
    pub fn new(root_path: &Path) -> Result<TomlConfig, Box<dyn Error>> {
        let settings_path = root_path.join("settings.toml");
        let mut file = File::open(settings_path)?;
        let mut settings = String::new();
        file.read_to_string(&mut settings)?;
        let settings: Settings = from_str(&settings)?;
        let mut users = HashMap::new();
        let mut tests = HashMap::new();
        for test in &settings.tests {
            tests.insert(test.caption.clone(), test.clone());
            for user in &test.allowed_users {
                if !users.contains_key(user) {
                    users.insert(user.clone(), HashSet::new());
                }
                users.get_mut(user).unwrap().insert(test.caption.clone());
            }
        }

        Ok(TomlConfig {
            settings,
            users,
            tests,
        })
    }
}

impl Config for TomlConfig {
    /// Существует ли пользователь?
    fn has_user(&self, username: &String) -> bool {
        self.users.contains_key(username)
    }

    /// Проверить валидность теста testname.
    fn has_test(&self, testname: &String) -> bool {
        self.tests.contains_key(testname)
    }

    /// Получить параметры теста testname.
    fn test_settings(&self, testname: &String) -> TestSettings {
        self.tests[testname].to_owned()
    }

    /// Получить описание теста.
    fn test_banner(&self, testname: &String) -> String {
        self.tests[testname].banner.clone()
    }

    /// Получить текст вопроса question_id теста testname.
    fn question(&self, testname: &String, question_id: usize) -> Question {
        unimplemented!("Нет чтения файлов тестов!");
        //self.tests[testname].questions[question_id].clone()
    }

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&self, testname: &String, question_id: usize) -> Answer {
        // TODO! Спарсить вопросы
        unimplemented!("Нет чтения файлов тестов!");
        //Answer { answers: vec![] }
    }

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&self, username: &String, testname: &String) -> bool {
        self.users[username].contains(testname)
    }

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&self, username: &String) -> Vec<String> {
        self.users[username].clone().into_iter().collect()
    }

    /// Получить параметры сервера.
    fn settings(&self) -> Settings {
        self.settings.clone()
    }
}
