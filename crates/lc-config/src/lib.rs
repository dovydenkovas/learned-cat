#![allow(unused)]

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
    rc::Rc,
};

use std::error::Error;

use parsetest::read_test;
use toml::from_str;

mod parsetest;

use lc_examiner::{
    schema::{Answer, Question},
    settings::{Settings, Test, TestSettings},
    Config,
};

#[derive(Debug, PartialEq, Clone)]
pub struct TomlConfig {
    settings: Settings,

    // Таблица пользователь: список доступных тестов.
    users: HashMap<String, HashSet<String>>,
    // Таблица тест: настройки.
    tests: HashMap<String, Test>,
    test_settings: HashMap<String, TestSettings>,
}

impl TomlConfig {
    pub fn new(root_path: &Path) -> Result<TomlConfig, Box<dyn Error>> {
        let settings_path = root_path.join("settings.toml");
        let mut file = File::open(settings_path)?;
        let mut settings = String::new();
        file.read_to_string(&mut settings)?;
        let mut settings: Settings = from_str(&settings)?;
        let mut users = HashMap::new();
        let mut tests = HashMap::new();
        let mut test_settings = HashMap::new();

        let path = root_path.join(&settings.tests_directory_path);
        for test in &settings.tests {
            let test_path = path.join(test.caption.clone() + ".md");
            let questions = read_test(&test_path);
            tests.insert(test.caption.clone(), questions);

            test_settings.insert(test.caption.clone(), test.clone());
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
            test_settings,
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
    fn test_settings(&self, testname: &String) -> Option<TestSettings> {
        if self.has_test(testname) {
            return Some(self.test_settings[testname].clone());
        }
        None
    }

    /// Получить описание теста.
    fn test_banner(&self, testname: &String) -> Option<String> {
        if self.has_test(testname) {
            return Some(self.tests[testname].banner.clone());
        }
        None
    }

    /// Получить текст вопроса question_id теста testname.
    fn question(&self, testname: &String, question_id: usize) -> Option<Question> {
        if self.has_test(testname) && question_id < self.tests[testname].questions.len() {
            return Some(self.tests[testname].questions[question_id].clone());
        }
        None
    }

    fn questions_count(&self, testname: &String) -> Option<usize> {
        if self.has_test(testname) {
            return Some(self.tests[testname].questions.len());
        }
        None
    }

    /// Получить ответы на вопрос question_id теста testname.
    fn answer(&self, testname: &String, question_id: usize) -> Option<Answer> {
        if self.has_test(testname) && question_id < self.tests[testname].questions.len() {
            return Some(
                self.tests[testname].questions[question_id]
                    .correct_answer
                    .clone(),
            );
        }
        None
    }

    /// Проверить доступность теста testname для пользователя username.
    fn has_access(&self, username: &String, testname: &String) -> bool {
        self.has_user(username) && self.users[username].contains(testname)
    }

    /// Получить список тестов, доступных пользователю username.
    fn user_tests_list(&self, username: &String) -> Vec<String> {
        if self.has_user(username) {
            return self.users[username].clone().into_iter().collect();
        }
        vec![]
    }

    /// Получить параметры сервера.
    fn settings(&self) -> Settings {
        self.settings.clone()
    }
}

#[cfg(test)]
mod tests {
    use lc_examiner::{
        schema::{Answer, Question},
        Config,
    };
    use std::path::Path;

    use crate::TomlConfig;

    fn load_config() -> TomlConfig {
        let path = Path::new("../../example-config/");
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
        let mut asd_tests = config.user_tests_list(&"asd".to_string());
        asd_tests.sort();
        assert_eq!(asd_tests, vec!["algo", "linux", "python"]);
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
            config.test_banner(&"python".to_string()).unwrap(),
            "Предполагается, что все используемые в примерах библиотеки были импортированы."
        );
        assert!(config.test_banner(&"astronomy".to_string()).is_none());
        let settings = config.test_settings(&"linux".to_string()).unwrap();
        assert_eq!(settings.caption, "linux");
        assert_eq!(settings.allowed_users, vec!["asd", "student"]);
        assert_eq!(settings.number_of_attempts, 3);
    }

    #[test]
    fn tests_questions() {
        let config = load_config();
        assert_eq!(config.questions_count(&"linux".to_string()).unwrap(), 6);
        assert_eq!(
            config.question(&"linux".to_string(), 0).unwrap(),
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
