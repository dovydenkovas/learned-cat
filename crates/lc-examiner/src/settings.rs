use crate::schema::Question;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub struct Test {
    pub banner: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct TestSettings {
    /// Basic info
    pub caption: String,

    /// Variant parameters
    #[serde(default)]
    pub questions_number: usize,

    #[serde(default)]
    pub test_duration_minutes: i64,

    #[serde(default)]
    pub number_of_attempts: u32,

    /// Castumization
    #[serde(default)]
    pub show_results: bool,

    #[serde(default)]
    pub allowed_users: Option<Vec<String>>,

    #[serde(default)]
    pub allowed_users_path: Option<String>,
}

impl std::default::Default for TestSettings {
    fn default() -> TestSettings {
        TestSettings {
            caption: "".to_string(),
            questions_number: 0,
            test_duration_minutes: 0,
            show_results: true,
            allowed_users: Some(vec![]),
            allowed_users_path: None,
            number_of_attempts: 1,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub tests_directory_path: String,

    #[serde(default)]
    pub result_path: String,

    #[serde(default)]
    pub server_address: String,

    #[serde(default)]
    #[serde(rename = "test")]
    pub tests: Vec<TestSettings>,

    #[serde(default)]
    pub log_level: String,
}

impl std::default::Default for Settings {
    fn default() -> Settings {
        Settings {
            tests_directory_path: "tests".to_string(),
            result_path: "results".to_string(),
            server_address: "127.0.0.1:65001".to_string(),
            tests: vec![],
            log_level: "debug".to_string(),
        }
    }
}
