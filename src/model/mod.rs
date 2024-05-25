use std::env::set_current_dir;
use std::error::Error;
use std::fs::File;
/// Содержит структуры тестов
use std::io::{Read, Write};
use std::path::Path;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use toml::from_str as from_toml;
use walkdir::WalkDir;

pub mod errors;
pub mod init;
pub mod parsetest;
use errors::{ModelError, ModelResult};
use parsetest::read_test;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Question {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Test {
    /// Basic info
    pub caption: String,

    #[serde(default)]
    pub banner: String,

    /// Variant parameters
    #[serde(default)]
    pub questions: Vec<Question>,

    pub questions_number: usize,
    pub test_duration_minutes: i64,
    pub number_of_attempts: usize,

    /// Castumization
    pub show_results: bool,

    pub allowed_users: Vec<String>,
}

impl std::default::Default for Test {
    fn default() -> Test {
        Test {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Variant {
    pub username: String,
    pub testname: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
    questions: Vec<Question>,
    answers: Vec<Vec<usize>>,
    current_question: usize,
    pub result: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Variants {
    pub variants: Vec<Variant>,
}

type TestResults = std::collections::hash_map::HashMap<String, Variants>;

#[derive(Deserialize, Debug)]
pub struct Model {
    #[serde(default)]
    tests_directory_path: String,

    #[serde(default)]
    pub result_path: String,

    #[serde(default)]
    server_address: String,

    #[serde(default)]
    #[serde(rename = "test")]
    tests: Vec<Test>,

    #[serde(default)]
    results: TestResults,
}

impl std::default::Default for Model {
    fn default() -> Model {
        Model {
            tests_directory_path: "tests".to_string(),
            result_path: "results".to_string(),
            server_address: "127.0.0.1:65001".to_string(),
            tests: vec![],
            results: std::collections::hash_map::HashMap::new(),
        }
    }
}

impl Model {
    pub fn new(mut settings: Model) -> Model {
        println!("* Чтение тестов: ");
        set_daemon_dir().expect("Error init and start server");
        // Read tests
        let quests_base_path = Path::new(&settings.tests_directory_path);
        for test in &mut settings.tests {
            let path = quests_base_path.join(Path::new(&(test.caption.to_owned() + ".md")));
            read_test(&path, test);
        }

        for test in &settings.tests {
            println!("  * {}", test.caption);
        }

        let results = load_results(&settings.result_path);
        settings.results = results;
        settings
    }

    pub fn get_server_address(&self) -> String {
        self.server_address.clone()
    }

    pub fn get_banner(&self, testname: &String) -> ModelResult<String> {
        let id = self.get_test_id_by_name(testname)?;
        Ok(self.tests[id].banner.clone())
    }

    pub fn is_allowed_user(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let id = self.get_test_id_by_name(testname)?;
        let test = &self.tests[id];
        Ok(test.allowed_users.contains(username))
    }

    pub fn start_test(&mut self, username: &String, testname: &String) -> ModelResult<String> {
        if !self.is_allowed_user(username, testname)? {
            return Err(ModelError::UserNotAllowed);
        }

        if self.is_user_done_test(username, testname)? {
            return Err(ModelError::TestIsDone);
        }

        if self.is_user_have_opened_variant(username, testname)? {
            return Ok(self.get_banner(testname)?);
        }

        let variant = self.generate_variant(username, testname)?;
        self.create_test_record(username, testname, variant);
        Ok(self.get_banner(testname)?)
    }

    /// Return [true] if user done the test.
    fn is_user_done_test(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + testname;
        if self.results.contains_key(&result_mark) {
            let id = self.get_test_id_by_name(testname)?;
            let test = &self.tests[id];
            if self.results.get(&result_mark).unwrap().variants.len() >= test.number_of_attempts {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_user_have_opened_variant(
        &self,
        username: &String,
        testname: &String,
    ) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + testname;
        if self.results.contains_key(&result_mark) {
            let variant = &self
                .results
                .get(&result_mark)
                .unwrap()
                .variants
                .last()
                .unwrap();
            return Ok(!variant.result.is_some());
        }
        Ok(false)
    }

    fn generate_variant(&self, username: &String, testname: &String) -> ModelResult<Variant> {
        let id = self.get_test_id_by_name(testname)?;
        let test = &self.tests[id];

        let questions: Vec<Question> = test
            .questions
            .choose_multiple(&mut rand::thread_rng(), test.questions_number)
            .cloned()
            .collect();

        Ok(Variant {
            username: username.clone(),
            testname: testname.clone(),
            timestamp: chrono::offset::Local::now(),
            questions,
            answers: vec![],
            current_question: 0,
            result: None,
        })
    }

    fn get_test_id_by_name(&self, testname: &String) -> ModelResult<usize> {
        for i in 0..self.tests.len() {
            if &self.tests[i].caption == testname {
                return Ok(i);
            }
        }

        Err(ModelError::TestNotExist(testname.clone()))
    }

    fn create_test_record(&mut self, username: &String, testname: &String, variant: Variant) {
        let result_mark = username.to_owned() + "@" + testname;
        if !self.results.contains_key(&result_mark) {
            self.results
                .insert(result_mark.clone(), Variants { variants: vec![] });
        }
        self.results
            .get_mut(&result_mark)
            .unwrap()
            .variants
            .push(variant);
    }

    pub fn get_avaliable_tests(&self, username: &String) -> ModelResult<Vec<(String, String)>> {
        let mut res: Vec<(String, String)> = vec![];
        for test in &self.tests {
            if test.allowed_users.contains(username) {
                let result = match self.get_result(username, &test) {
                    Ok(result) => result,
                    Err(_) => "".to_string(),
                };
                res.push((test.caption.clone(), result))
            }
        }
        Ok(res)
    }

    fn is_test_time_is_over(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + testname;
        if self.results.contains_key(&result_mark) {
            let id = self.get_test_id_by_name(testname)?;
            let test = &self.tests[id];
            let variant = &self
                .results
                .get(&result_mark)
                .unwrap()
                .variants
                .last()
                .unwrap();

            if (chrono::Local::now() - variant.timestamp)
                > chrono::Duration::new(test.test_duration_minutes * 60, 0).unwrap()
            {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    pub fn get_next_question(
        &mut self,
        username: &String,
        testname: &String,
    ) -> ModelResult<Question> {
        let result_mark = username.to_owned() + "@" + &testname;

        if !self.results.contains_key(&result_mark) {
            return Err(ModelError::VariantNotExist(result_mark.clone()));
        }

        if self.results[&result_mark]
            .variants
            .last()
            .unwrap()
            .result
            .is_some()
        {
            return Err(ModelError::TestIsDone);
        }

        if self.is_test_time_is_over(username, testname)? {
            self.done_test(username, testname);
            return Err(ModelError::TestIsDone);
        }

        let id = self.results[&result_mark]
            .variants
            .last()
            .unwrap()
            .current_question;
        return Ok(self.results[&result_mark]
            .variants
            .last()
            .unwrap()
            .questions[id]
            .clone());
    }

    pub fn is_next_question(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            Ok(self.results[&result_mark]
                .variants
                .last()
                .unwrap()
                .current_question
                < self.results[&result_mark]
                    .variants
                    .last()
                    .unwrap()
                    .questions
                    .len())
        } else {
            Ok(false)
        }
    }

    pub fn put_answer(
        &mut self,
        username: &String,
        testname: &String,
        answer: &Vec<usize>,
    ) -> ModelResult<()> {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            let variant = &mut self
                .results
                .get_mut(&result_mark)
                .unwrap()
                .variants
                .last_mut()
                .unwrap();
            if variant.result.is_none() {
                variant.answers.push(answer.clone());
                variant.current_question += 1;
                if variant.current_question == variant.questions.len() {
                    self.done_test(username, testname)
                }
                return Ok(());
            }
            return Err(ModelError::TestIsDone);
        }
        Err(ModelError::VariantNotExist(result_mark.clone()))
    }

    fn get_result(&self, username: &String, test: &Test) -> ModelResult<String> {
        let result_mark = username.to_owned() + "@" + &test.caption;
        if self.results.contains_key(&result_mark) {
            let mut result = 0;
            let mut has_result = false;
            for variant in &self.results[&result_mark].variants {
                match variant.result {
                    Some(res) => {
                        result = std::cmp::max(result, res);
                        has_result = true;
                    }
                    None => (),
                }
            }

            if has_result {
                Ok(result.to_string())
            } else {
                Err(ModelError::ResultNotExist(result_mark.clone()))
            }
        } else {
            Err(ModelError::VariantNotExist(result_mark.clone()))
        }
    }

    pub fn get_result_by_testname(
        &self,
        username: &String,
        testname: &String,
    ) -> ModelResult<String> {
        let id = self.get_test_id_by_name(testname)?;
        let test = &self.tests[id];
        Ok(self.get_result(username, test)?)
    }

    /*pub fn collect_done_tests(&mut self) {
        // TODO Add result collector: Automatically end tests with time is over
        println!("Starting collector of done tests");
    }*/

    fn done_test(&mut self, username: &String, testname: &String) {
        self.calculate_mark(username, testname);
        self.save_result(username, testname);
    }

    fn calculate_mark(&mut self, username: &String, testname: &String) {
        // TODO
        let result_mark = username.to_owned() + "@" + testname;
        if self.results.contains_key(&result_mark) {
            let variant = &mut self
                .results
                .get_mut(&result_mark)
                .unwrap()
                .variants
                .last_mut()
                .unwrap();
            let mut result = 0;
            for i in 0..variant.questions.len() {
                variant.answers[i].sort();
                if variant.questions[i].correct_answers == variant.answers[i] {
                    result += 1;
                }
            }
            variant.result = Some(result);
        }
    }

    fn save_result(&mut self, username: &String, testname: &String) {
        let result_mark = username.to_owned() + "@" + testname;

        let filename = self.result_path.to_owned() + "/" + &result_mark + ".toml";
        println!("{filename}");
        let mut ofile = File::create(filename).expect("Не могу открыть файл");

        let result = &self.results[&result_mark];
        println!("{:?}", result);

        let out = toml::to_string(&result).expect("Не могу экспортировать файлы");
        println!("{out}");
        let _ = ofile.write(out.as_bytes());
    }

    /*pub fn get_results(&self) -> TestResults {
        self.results.clone()
    }*/
}

pub fn read_settings() -> Result<Model, Box<dyn Error>> {
    let settings_path = get_daemon_dir_path() + "/settings.toml";
    let mut file = File::open(settings_path)?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}

pub fn load_results(result_path: &String) -> TestResults {
    println!("* Чтение результатов:");
    let _ = std::fs::create_dir(result_path);

    let mut results = std::collections::hash_map::HashMap::new();

    for entry in WalkDir::new(result_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            println!("   * {}", entry.path().display());
            match load_results_from_file(entry.path()) {
                Ok(result) => {
                    let key = entry
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    results.insert(key, result);
                }
                Err(e) => println!("{e}"),
            }
        }
    }

    results
}

fn load_results_from_file(result_filename: &Path) -> Result<Variants, Box<dyn Error>> {
    let mut file = File::open(result_filename)?;
    let mut results_string = String::new();
    file.read_to_string(&mut results_string)?;
    Ok(from_toml(&results_string)?)
}

fn set_daemon_dir() -> Result<(), Box<dyn Error>> {
    let daemon_path = get_daemon_dir_path();
    let root = std::path::Path::new(&daemon_path); // TODO
    if set_current_dir(&root).is_err() {
        eprintln!(
            "Ошибка доступа к каталогу сервера {}.",
            root.to_str().unwrap()
        );
        eprintln!("Проверьте, что каталог существует, и у процесса есть у нему доступ.");
        return Err(Box::new(std::fmt::Error));
    }
    Ok(())
}

pub fn get_daemon_dir_path() -> String {
    "/home/asd/code/desktop/sshtest/sshtest-dir".to_string()
}
