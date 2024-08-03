use std::collections::hash_map::HashMap;
use std::env::set_current_dir;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use toml::from_str as from_toml;
use walkdir::WalkDir;

pub mod errors;
pub mod init;
pub mod parsetest;
use errors::{ModelError, ModelResult};
use parsetest::read_test;

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Question {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Test {
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
    pub test_duration_seconds: i64,

    #[serde(default)]
    pub number_of_attempts: usize,

    /// Castumization
    #[serde(default)]
    pub show_results: bool,

    #[serde(default)]
    pub allowed_users: Vec<String>,
}

impl std::default::Default for Test {
    fn default() -> Test {
        Test {
            caption: "".to_string(),
            banner: "".to_string(),
            questions: vec![],
            questions_number: 0,
            test_duration_seconds: 0,
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
    pub timestamp: Option<chrono::DateTime<chrono::Local>>,
    questions: Vec<Question>,
    answers: Vec<Vec<usize>>,
    current_question: Option<usize>,
    pub result: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Variants {
    pub variants: Vec<Variant>,
}

type TestResults = std::collections::hash_map::HashMap<String, Variants>;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Settings {
    #[serde(default = "Settings::default_tests_directory_path")]
    pub tests_directory_path: String,

    #[serde(default = "Settings::default_result_path")]
    pub result_path: String,

    #[serde(default = "Settings::default_server_address")]
    pub server_address: String,

    #[serde(default = "Settings::default_tests")]
    #[serde(rename = "test")]
    pub tests: Vec<Test>,

    #[serde(default = "Settings::default_new_file_permissions")]
    pub new_file_permissions: u32,

    #[serde(default = "Settings::default_verifier_interval_seconds")]
    pub verifier_interval_seconds: u64,
}

impl Settings {
    fn default_tests_directory_path() -> String {
        "tests".to_string()
    }

    fn default_result_path() -> String {
        "results".to_string()
    }

    fn default_server_address() -> String {
        "127.0.0.1:65001".to_string()
    }

    fn default_tests() -> Vec<Test> {
        vec![]
    }

    fn default_new_file_permissions() -> u32 {
        0o640
    }

    fn default_verifier_interval_seconds() -> u64 {
        5
    }
}

impl std::default::Default for Settings {
    fn default() -> Settings {
        Settings {
            tests_directory_path: Settings::default_tests_directory_path(),
            result_path: Settings::default_result_path(),
            server_address: Settings::default_server_address(),
            tests: Settings::default_tests(),
            new_file_permissions: Settings::default_new_file_permissions(),
            verifier_interval_seconds: Settings::default_verifier_interval_seconds(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Model {
    result_path: PathBuf,
    server_address: String,
    tests: HashMap<String, Test>,
    results: TestResults,
    new_file_permissions: u32,
    root_path: PathBuf,
}

impl Model {
    pub fn begin<P: AsRef<Path>>(settings: Settings, root_path: P) -> Arc<Mutex<Model>> {
        println!("* Чтение тестов: ");
        set_daemon_dir(&root_path).expect("Error init and start server");

        // Read tests
        let quests_base_path = Path::new(&settings.tests_directory_path);
        let mut tests: HashMap<String, Test> = HashMap::new();
        for mut test in settings.tests {
            let path = quests_base_path.join(Path::new(&(test.caption.to_owned() + ".md")));
            read_test(&path, &mut test);
            tests.insert(test.caption.clone(), test);
        }

        for test in tests.keys() {
            println!("  * {}", test);
        }

        let results = load_results(&settings.result_path);

        let model = Arc::new(Mutex::new(Model {
            server_address: settings.server_address,
            result_path: PathBuf::from(settings.result_path),
            tests,
            results,
            new_file_permissions: settings.new_file_permissions,
            root_path: root_path.as_ref().into(),
        }));

        let arc_model = Arc::clone(&model);
        std::thread::spawn(move || test_collector(arc_model, settings.verifier_interval_seconds));

        model
    }

    pub fn get_server_address(&self) -> String {
        self.server_address.clone()
    }

    pub fn get_banner(&self, testname: &String) -> ModelResult<String> {
        Ok(self
            .tests
            .get(testname)
            .unwrap_or(&Test::default())
            .banner
            .clone())
    }

    pub fn is_allowed_user(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let test = match &self.tests.get(testname) {
            Some(v) => v.to_owned(),
            None => return Ok(false),
        };
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
            return Err(ModelError::TestIsOpened(username.clone(), testname.clone()));
        }

        let variant = self.generate_variant(username, testname)?;
        self.create_test_record(username, testname, variant);
        Ok(self.get_banner(testname)?)
    }

    /// Return [true] if user done the test.
    fn is_user_done_test(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + testname;
        if self.results.contains_key(&result_mark) {
            let test = &self
                .tests
                .get(testname)
                .ok_or(ModelError::TestNotExist(testname.to_string()))?;
            if self.results[&result_mark].variants.len() >= test.number_of_attempts
                && self.results[&result_mark]
                    .variants
                    .last()
                    .unwrap()
                    .result
                    .is_some()
            {
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
        let test = &self
            .tests
            .get(testname)
            .ok_or(ModelError::TestNotExist(testname.to_string()))?;

        let questions: Vec<Question> = test
            .questions
            .choose_multiple(&mut rand::thread_rng(), test.questions_number)
            .cloned()
            .collect();

        Ok(Variant {
            username: username.clone(),
            testname: testname.clone(),
            timestamp: None,
            questions,
            answers: vec![],
            current_question: None,
            result: None,
        })
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
        for test in self.tests.values() {
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
            let test = &self
                .tests
                .get(testname)
                .ok_or(ModelError::TestNotExist(testname.to_string()))?;
            let variant = &self
                .results
                .get(&result_mark)
                .unwrap()
                .variants
                .last()
                .unwrap();

            if variant.timestamp.is_none() {
                return Ok(false);
            }

            if (chrono::Local::now() - variant.timestamp.unwrap())
                > chrono::Duration::new(test.test_duration_seconds, 0).unwrap()
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

        let variant = self
            .results
            .get_mut(&result_mark)
            .unwrap()
            .variants
            .last_mut()
            .unwrap();
        if variant.timestamp.is_none() {
            variant.timestamp = Some(chrono::offset::Local::now());
        }

        let mut id = variant.current_question;
        if variant.current_question.is_none() {
            variant.current_question = Some(0);
            id = Some(0);
        }
        return Ok(variant.questions[id.unwrap()].clone());
    }

    pub fn is_next_question(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            let current_question = self.results[&result_mark]
                .variants
                .last()
                .unwrap()
                .current_question;
            if current_question.is_none() {
                return Ok(true);
            }
            Ok(current_question.unwrap()
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
                if variant.current_question.is_none() {
                    return Err(ModelError::TestNotStarted);
                }
                variant.current_question = Some(variant.current_question.unwrap() + 1);
                if variant.current_question.unwrap() == variant.questions.len() {
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
            let mut result = String::new();
            let mut best_mark = 0;
            let mut has_result = false;
            for variant in &self.results[&result_mark].variants {
                match variant.result {
                    Some(res) => {
                        if test.show_results {
                            best_mark = std::cmp::max(best_mark, res);
                            result = best_mark.to_string();
                        } else {
                            result = "Тест завершен.".to_string();
                        }
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
        let test = &self
            .tests
            .get(testname)
            .ok_or(ModelError::TestNotExist(testname.to_string()))?;
        Ok(self.get_result(username, test)?)
    }

    fn done_test(&mut self, username: &String, testname: &String) {
        self.calculate_mark(username, testname);
        self.save_result(username, testname);
    }

    fn calculate_mark(&mut self, username: &String, testname: &String) {
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
            for i in 0..variant.answers.len() {
                if i >= variant.questions.len() {
                    break;
                }
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

        let filename = self
            .root_path
            .join(&self.result_path)
            .join(&result_mark)
            .with_extension("toml");
        let mut ofile = File::create(&filename).expect("Не могу открыть файл результата");
        let path = Path::new(&filename);
        let result = &self.results[&result_mark];

        let out = toml::to_string(&result).expect("Не могу экспортировать файлы результата");
        ofile
            .write(out.as_bytes())
            .expect("Ошибка записи результата");

        init::chmod(path, self.new_file_permissions);
    }
}

pub fn read_settings<P: AsRef<Path>>(path: P) -> Result<Settings, Box<dyn Error>> {
    let settings_path = path.as_ref().join("settings.toml");
    let mut file = File::open(settings_path)?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}

pub fn load_results<P: AsRef<Path>>(result_path: P) -> TestResults {
    println!("* Чтение результатов:");
    let _ = std::fs::create_dir(&result_path);

    let mut results = std::collections::hash_map::HashMap::new();

    for entry in WalkDir::new(&result_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
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

fn test_collector(model: Arc<Mutex<Model>>, verifier_interval_seconds: u64) {
    loop {
        {
            println!("Проверка завершения времени тестирования");
            let mut done_tests: Vec<(String, String)> = vec![];
            {
                let model = model.lock().unwrap();
                for result in model.results.values().by_ref() {
                    if result.variants.len() > 0 && result.variants.last().unwrap().result.is_none()
                    {
                        let username = &result.variants.last().unwrap().username;
                        let testname = &result.variants.last().unwrap().testname;
                        let res = model.is_test_time_is_over(username, &testname);
                        if res.is_ok() && res.unwrap() {
                            println!("{username}, {testname}");
                            done_tests.push((username.clone(), testname.clone()));
                        }
                    }
                }
            }

            for (username, testname) in done_tests {
                model.lock().unwrap().done_test(&username, &testname);
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(verifier_interval_seconds));
    }
}

fn load_results_from_file(result_filename: &Path) -> Result<Variants, Box<dyn Error>> {
    let mut file = File::open(result_filename)?;
    let mut results_string = String::new();
    file.read_to_string(&mut results_string)?;
    Ok(from_toml(&results_string)?)
}

fn set_daemon_dir<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    if set_current_dir(path.as_ref()).is_err() {
        eprintln!(
            "Ошибка доступа к каталогу сервера {}.",
            path.as_ref().to_str().unwrap()
        );
        eprintln!("Проверьте, что каталог существует, и у процесса есть у нему доступ.");
        return Err(Box::new(std::fmt::Error));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{read_settings, Model};
    use crate::{
        get_daemon_dir_path,
        model::{init::init_server, Settings, Test},
    };
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };

    fn free_resource<S: AsRef<str>>(name: S) {
        let dir = std::env::temp_dir();
        let path = dir.join(name.as_ref());
        let _ = std::fs::remove_dir_all(&path);
    }

    fn get_test_model<S: AsRef<str>>(name: S) -> Arc<Mutex<Model>> {
        let dir = std::env::temp_dir();
        let path = dir.join(name.as_ref());
        init_server(&path);
        let settings = read_settings(&path).unwrap();

        Model::begin(settings, path)
    }

    /*fn new_tempfile<S: AsRef<str>>(content: S) -> std::path::PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "{}",
            std::time::Instant::now().elapsed().as_nanos()
        ));
        let mut file = std::fs::File::create_new(&path).unwrap();
        file.write(content.as_ref().as_bytes());
        path
    }

    fn rm_tempfile<S: AsRef<str>>(path: S) {
    std::fs::remove_file(std::path::Path::new(path.as_ref())).unwrap();
    }*/

    #[test]
    fn get_daemon_dir_test() {
        assert_eq!(
            get_daemon_dir_path(),
            std::path::Path::new("/opt/learned-cat/")
        );

        std::env::set_var("LEARNED_CAT_PATH", "/usr/share/lc");

        assert_eq!(get_daemon_dir_path(), std::path::Path::new("/usr/share/lc"));
    }

    /*#[test]
    fn test_collector_test() {
        // test_collector(model, verifier_interval_seconds)
        // TODO
        }*/

    #[test]
    fn read_settings_test() {
        assert!(read_settings("qwerty").is_err());
        assert!(read_settings("").is_err());
        assert!(read_settings("/etc/passwd").is_err());

        let model = get_test_model("model-settings");
        let settings_path = std::env::temp_dir().join("model-settings");
        let settings = read_settings(&settings_path).unwrap();

        assert_eq!(
            Settings {
                tests_directory_path: "tests".to_string(),
                result_path: "results".to_string(),
                server_address: "127.0.0.1:65001".to_string(),
                tests: vec![
                    Test {
                        caption: "linux".to_string(),
                        banner: "".to_string(),
                        questions: vec![],
                        questions_number: 1,
                        test_duration_seconds: 5,
                        number_of_attempts: 1,
                        show_results: true,
                        allowed_users: vec!["student1".to_string(), "student2".to_string()]
                    },
                    Test {
                        caption: "python".to_string(),
                        banner: "".to_string(),
                        questions: vec![],
                        questions_number: 2,
                        test_duration_seconds: 1,
                        number_of_attempts: 0,
                        show_results: false,
                        allowed_users: vec!["student2".to_string()]
                    }
                ],
                new_file_permissions: 420,
                verifier_interval_seconds: 1
            },
            settings,
            "Несоответсвие параметров конфига значениям по умолчанию"
        );

        let mut file = std::fs::File::create(&settings_path.join("settings.toml")).unwrap();
        file.write_all(b"[wrong] \nwrong = 42").unwrap();
        file.flush().unwrap();
        drop(file);
        let settings = read_settings(settings_path).unwrap();
        assert_eq!(Settings::default(), settings);

        drop(model);
        free_resource("model-settings");
    }
    /*
    #[test]
    fn calculate_mark_test() {
        // TODO
    }

    #[test]
    fn put_answer() {
        // TODO
    }

    #[test]
    fn get_next_question() {
        // TODO
        }*/
}
