use crate::schema::Question;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TestSettings {
    /// Basic info
    pub caption: String,

    #[serde(default)]
    pub banner: String,

    /// Variant parameters
    #[serde(default)]
    pub questions: Vec<Question>,

    #[serde(default)]
    pub questions_number: usize,

    #[serde(default)]
    pub test_duration_minutes: i64,

    #[serde(default)]
    pub number_of_attempts: usize,

    /// Castumization
    #[serde(default)]
    pub show_results: bool,

    #[serde(default)]
    pub allowed_users: Vec<String>,
}

impl std::default::Default for TestSettings {
    fn default() -> TestSettings {
        TestSettings {
            caption: "".to_string(),
            banner: "".to_string(),
            questions: vec![],
            questions_number: 0,
            test_duration_minutes: 0,
            show_results: true,
            allowed_users: vec![],
            number_of_attempts: 1,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
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
    pub new_file_permissions: u32,
}

impl std::default::Default for Settings {
    fn default() -> Settings {
        Settings {
            tests_directory_path: "tests".to_string(),
            result_path: "results".to_string(),
            server_address: "127.0.0.1:65001".to_string(),
            tests: vec![],
            new_file_permissions: 0o640,
        }
    }
}
