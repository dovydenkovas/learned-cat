#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
};

use std::error::Error;

use toml::from_str;

mod parsetest;

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

#[cfg(test)]
mod tests {
    use learned_cat_interfaces::{
        schema::{Answer, Question},
        Config,
    };
    use std::path::Path;

    use crate::TomlConfig;

    fn load_config() -> TomlConfig {
        let path = Path::new("../example-config/settings.toml");
        let conf = TomlConfig::new(path);
        assert!(conf.is_ok());
        conf.unwrap()
    }

    #[test]
    fn check_main_settings() {
        let config = load_config();
        assert_eq!(config.settings().result_path, "marks.db");
        assert_eq!(config.settings().tests_directory_path, "tests");
        assert_eq!(config.settings().server_address, "127.0.0.1:8080");
    }

    #[test]
    fn users() {
        let config = load_config();
        assert!(config.has_user(&"student".to_string()));
        assert!(!config.has_user(&"tux".to_string()));
        assert_eq!(
            config.user_tests_list(&"asd".to_string()),
            vec!["linux", "python"]
        );
        assert_eq!(
            config.user_tests_list(&"student".to_string()),
            vec!["linux"]
        );
        assert_eq!(
            config.user_tests_list(&"tux".to_string()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn test_info() {
        let config = load_config();
        assert!(config.has_test(&"python".to_string()));
        assert!(!config.has_test(&"astronomy".to_string()));
        assert_eq!(
            config.test_banner(&"python".to_string()),
            "Предполагается, что все используемые в примерах библиотеки были импортированы."
        );
        assert_eq!(config.test_banner(&"astronomy".to_string()), "");
        let settings = config.test_settings(&"linux".to_string());
        assert_eq!(settings.caption, "linux");
        assert_eq!(
            settings.banner,
            "Тест на знание основных иструментов Linux."
        );
        assert_eq!(settings.allowed_users, vec!["asd", "student"]);
        assert_eq!(settings.number_of_attempts, 3);
    }

    #[test]
    fn tests_questions() {
        let config = load_config();
        assert_eq!(
            config.test_settings(&"linux".to_string()).questions.len(),
            6
        );

        assert_eq!(
            config.question(&"linux".to_string(), 0),
            Question {
                question: "Что делает утилита cat?".to_string(),
                answers: vec![
                    "Вызывает кота, который бегает за курсором мыши".to_string(),
                    "Выводит содержимое файла".to_string(),
                    "Это пакетный менеджер, позволяет устанавливать программы".to_string(),
                    "Явно ничего хорошего".to_string(),
                    "Такой команды нет".to_string()
                ],
                correct_answer: Answer::new(vec![1]),
            }
        );
    }
}
