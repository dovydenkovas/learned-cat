/// Содержит структуры тестов
use std::io::Read;
use std::fs::File;
use std::error::Error;
use std::path::Path;

use toml::from_str as from_toml;
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;


pub mod parsetest;
pub mod init;
pub mod errors;
use parsetest::read_test;
use errors::{ModelResult, ModelError};


#[derive(Debug, Deserialize, Clone, Serialize)]
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
    pub test_duration_minutes: i64,
    pub number_of_attempts: usize,

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
            test_duration_minutes: 0,
            show_results: true,
            allowed_users: vec![], 
            number_of_attempts: 1,
        }
    }
}



#[derive(Debug, Deserialize, Serialize)]
struct Variant {
    username: String,
    testname: String,
    timestamp: chrono::DateTime<chrono::Local>,
    questions: Vec<Question>,
    answers: Vec<Vec<u8>>,
    current_question: usize,
    result: Option<usize>, 
}

type TestResults = std::collections::hash_map::HashMap<String, Vec<Variant>>;

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
    results: TestResults, 
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
  

    pub fn start_test(&mut self, username: &String, testname: &String) -> ModelResult<String> {
        if !self.is_allowed_user(username, testname)? {
            return Err(ModelError::UserNotAllowed)
        }
        
        if self.is_user_have_opened_variant(username, testname)? {
            return Err(ModelError::TestIsOpened(testname.clone()))
        }

        if self.is_user_done_test(username, testname)? {
            return Err(ModelError::TestIsDone) 
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
            if self.results.get(&result_mark).unwrap().len() >= test.number_of_attempts {
                return Ok(true) 
            }
        }
        Ok(false) 
    }


    pub fn is_user_have_opened_variant(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + testname;
        if self.results.contains_key(&result_mark) {
            let variant = &self.results.get(&result_mark).unwrap().last().unwrap();
            return Ok(variant.result.is_some());
        }
        Ok(false)
    }
 

    fn generate_variant(&self, username: &String, testname: &String) -> ModelResult<Variant> {
        let id = self.get_test_id_by_name(testname)?; 
        let test = &self.tests[id];
        
        let questions: Vec<Question> = test.questions
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
            result: None 
        })
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
        let result_mark = username.to_owned() + "@" + testname;
        if !self.results.contains_key(&result_mark) {
            self.results.insert(result_mark.clone(), vec![]);
        }
        self.results.get_mut(&result_mark).unwrap().push(variant); 
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
            let variant = & self.results.get(&result_mark).unwrap().last().unwrap();

            if (chrono::Local::now() - variant.timestamp) > 
                chrono::Duration::new(test.test_duration_minutes * 60, 0).unwrap() {
                return Ok(true) 
            }
        }
        return Ok(false);
    }
   

    pub fn get_next_question(&mut self, username: &String, testname: &String) -> ModelResult<Question> {
        let result_mark = username.to_owned() + "@" + &testname;    
        
        if !self.results.contains_key(&result_mark) {
            return Err(ModelError::VariantNotExist(result_mark.clone()))
        }

        if self.results[&result_mark].last().unwrap().result.is_some() {
            return Err(ModelError::TestIsDone) 
        }
        
        if self.is_test_time_is_over(username, testname)? {
            self.done_test(username, testname);
            return Err(ModelError::TestIsDone)
        }

        let id = self.results[&result_mark].last().unwrap().current_question;
        return Ok(self.results[&result_mark].last().unwrap().questions[id].clone()); 
    }

    pub fn is_next_question(&self, username: &String, testname: &String) -> ModelResult<bool> {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            Ok(self.results[&result_mark].last().unwrap().current_question 
               < self.results[&result_mark].last().unwrap().questions.len())
        } else {
           Ok(false)
        }
    }

    pub fn put_answer(&mut self, username: &String, testname: &String, answer: &Vec<u8>) -> ModelResult<()> {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            let variant = &mut self.results.get_mut(&result_mark).unwrap().last_mut().unwrap();
            if variant.result.is_none() {
                variant.answers.push(answer.clone()); 
                variant.current_question += 1;
                return Ok(())
            } 
            return Err(ModelError::TestIsDone);
        }
        Err(ModelError::VariantNotExist(result_mark.clone()))
    }


    fn get_result(&self, username: &String, test: &Test) -> ModelResult<String> {
        let result_mark = username.to_owned() + "@" + &test.caption;
        if self.results.contains_key(&result_mark) {
            match self.results[&result_mark].last().unwrap().result {
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

    pub fn collect_done_tests(&mut self) {
        // TODO Add result collector: Automatically end tests with time is over
        println!("Starting collector of done tests");
    }

    fn done_test(&mut self, username: &String, testname: &String) {
        // TODO done test and calculate mark 
    }
}


fn read_settings() -> Result<Model, Box<dyn Error>> {
    let mut file = File::open("settings.toml")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}


/// TODO load results
fn load_results(result_path: &String) -> TestResults {
    std::collections::hash_map::HashMap::new()
}



