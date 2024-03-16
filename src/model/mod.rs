/// Содержит структуры тестов
use std::io::Read;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::cmp;

use toml::from_str as from_toml;
use serde::Deserialize;


pub mod parsetest;
pub mod init;
pub mod errors;
use parsetest::read_test;
use errors::{ModelResult, ModelError};


#[derive(Debug, Deserialize, Clone)]
pub struct Question {
    pub question: String, 
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>
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
    pub shuffle_questions: bool,
    pub test_duration_minutes: u16,
    pub number_of_attempts: u16,

    /// Castumization 
    pub show_results: bool,

    pub allowed_users: Vec<String>
}


impl std::default::Default for Test {
    fn default() -> Test {
        Test { 
            caption: "".to_string(), 
            banner: "".to_string(),
            questions: vec![], 
            questions_number: 0,
            shuffle_questions: false, 
            test_duration_minutes: 0,
            show_results: true,
            allowed_users: vec![], 
            number_of_attempts: 1,
        }
    }
}



#[derive(Debug, Deserialize)]
struct Variant {
    questions: Vec<Question>,
    answers: Vec<Vec<u8>>,
    current_question: usize,
    result: Option<usize>, 
}


#[derive(Deserialize, Debug)]
pub struct Model {
    #[serde(default)]
    tests_directory_path: String,

    #[serde(default)]
    result_path: String,
    
    #[serde(default)]
    server_address: String, 

    #[serde(default)]
    #[serde(rename="test")]
    tests: Vec<Test>,
    
    #[serde(default)]
    results: std::collections::hash_map::HashMap<String, Variant>
}

impl std::default::Default for Model {
    fn default() -> Model {
        Model { 
            tests_directory_path: "tests".to_string(),
            result_path: "results".to_string(),
            server_address: "127.0.0.1:65001".to_string(),
            tests: vec![],
            results:  std::collections::hash_map::HashMap::new()
        }
    }
}


// TODO save result on test done 
impl Model {
    pub fn new() -> Model {
        println!("* Чтение файла конфигурации ");
        let mut settings = read_settings().expect("Не могу прочитать файл конфигурации settings.json.");
        println!("* Чтение тестов: ");

        // Read tests
        let quests_base_path = Path::new(&settings.tests_directory_path);
        for test in &mut settings.tests {
            let path =  quests_base_path.join(Path::new(&(test.caption.to_owned() + ".md")));
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
  

    pub fn start_test(&mut self, username: &String, test: &String) -> ModelResult<String> {
        // auth
        if self.is_user_done_test(username, test) {
            return Err(ModelError::UserNotAllowed); 
        }

        // TODO check number_of_attempts 
        // TODO generate questions
        let variant = self.generate_variant(&test)?;
        self.create_test_record(username, test, variant);
        Ok(self.get_banner(test)?)
    }
 

    fn generate_variant(&self, testname: &String) -> ModelResult<Variant> {
        // TODO select n questions 
        // TODO shuffle 
        let id = self.get_test_id_by_name(testname)?; 
        let test = &self.tests[id];
        
        let mut questions: Vec<Question> = vec![];
        for i in 0..cmp::max(0, cmp::min(test.questions_number, test.questions.len())) {
            questions.push(test.questions[i].clone()); 
        }

        Ok(Variant {questions, answers: vec![], current_question: 0, result: None })
    }


    fn get_test_id_by_name(&self, testname: &String) -> ModelResult<usize> {
        for i in 0..self.tests.len() {
            if &self.tests[i].caption == testname {
                return Ok(i) 
            }
        }

        Err(ModelError::TestNotExist(testname.clone()))
    }


    fn create_test_record(&mut self, username: &String, testname: &String, variant: Variant) {
        self.results.insert(username.to_owned() + "@" + testname, variant); 
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

    /// Return [true] if user done the test.
    fn is_user_done_test(&self, _username: &String, _test: &String) -> bool {
        // TODO is user done test (all attempts)
        false 
    }
    
    /// TODO Add result collector: Automatically end tests with time is over

    pub fn get_next_question(&self, username: &String, testname: &String) -> ModelResult<Question> {
        let result_mark = username.to_owned() + "@" + &testname;
        // TODO time of test not over
        if !self.results.contains_key(&result_mark) {
            return Err(ModelError::VariantNotExist(result_mark.clone()))
        }

        if self.results[&result_mark].result.is_some() {
            return Err(ModelError::TestIsDone) 
        }

        let id = self.results[&result_mark].current_question;
        return Ok(self.results[&result_mark].questions[id].clone()); 
    }

    pub fn is_next_question(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + &testname;
        // TODO check time of test is over 
        if self.results.contains_key(&result_mark) {
            Ok(self.results[&result_mark].current_question < self.results[&result_mark].questions.len())
        } else {
           Ok(false)
        }
    }

    pub fn put_answer(&mut self, username: &String, testname: &String, answer: &Vec<u8>) -> ModelResult<()> {
        let result_mark = username.to_owned() + "@" + &testname;
        // TODO check test not done 
        if self.results.contains_key(&result_mark) {
            self.results.get_mut(&result_mark).unwrap().answers.push(answer.clone()); 
            self.results.get_mut(&result_mark).unwrap().current_question += 1;
            return Ok(())
        }
        Err(ModelError::VariantNotExist(result_mark.clone()))
    }


    fn get_result(&self, username: &String, test: &Test) -> ModelResult<String> {
        let result_mark = username.to_owned() + "@" + &test.caption;
        if self.results.contains_key(&result_mark) {
            match self.results[&result_mark].result {
                Some(result) => Ok(result.to_string()),
                None => Err(ModelError::ResultNotExist(result_mark.clone()))
            }
            
        } else {
            Err(ModelError::VariantNotExist(result_mark.clone()))
        }
    }


    pub fn get_result_by_testname(&self, username: &String, testname: &String) -> ModelResult<String> {
        let id = self.get_test_id_by_name(testname)?; 
        let test = &self.tests[id]; 
        Ok(self.get_result(username, test)?)
    }
}


fn read_settings() -> Result<Model, Box<dyn Error>> {
    let mut file = File::open("settings.toml")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}


/// TODO load results
fn load_results(result_path: &String) -> std::collections::hash_map::HashMap<String, Variant> {
    std::collections::hash_map::HashMap::new()
}



