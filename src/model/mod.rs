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
use parsetest::read_test;



#[derive(Deserialize, Debug)]
struct Settings {
    #[serde(default)]
    tests_directory_path: String,

    #[serde(default)]
    tests: Vec<Test>
}

impl std::default::Default for Settings {
    fn default() -> Settings {
        Settings { 
            tests_directory_path: "tests".to_string(),
            tests: vec![]
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct Test {
    /// Basic info
    pub caption: String,
    pub banner: String,
    
    /// Variant parameters
    pub questions: Vec<Question>,
    pub question_number: usize,
    pub shuffle: bool,
    pub test_duration: u16,
    
    /// Castumization 
    pub show_results: bool,

    allowed_users: Vec<String>
}


impl std::default::Default for Test {
    fn default() -> Test {
        Test { 
            caption: "".to_string(), 
            banner: "".to_string(),
            questions: vec![], 
            question_number: 0,
            shuffle: false, 
            test_duration: 0,
            show_results: true,
            allowed_users: vec![]
        }
    }
}


#[derive(Debug, Deserialize, Clone)]
pub struct Question {
    pub question: String, 
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>
}


//#[derive(Debug)]
struct Variant {
    questions: Vec<Question>,
    answers: Vec<Vec<u8>>,
    current_question: usize,
    result: usize, 
}


pub struct Model {
    settings: Settings,
    results: std::collections::hash_map::HashMap<String, Variant> 
}


impl Model {
    pub fn new() -> Model {
        println!("* Чтение файла конфигурации ");
        let mut settings = read_settings().expect("Не могу прочитать файл конфигурации settings.json.");
        println!("* Чтение тестов: ");
       
        // Read tests
        let quests_base_path = Path::new(&settings.tests_directory_path);
        for test in &mut settings.tests {
            let path =  quests_base_path.join(Path::new(&(test.caption)));
            read_test(&path, test);
        }
                
        for test in &settings.tests {
            println!("  * {}", test.caption);
        }
        
        let results = std::collections::hash_map::HashMap::new();
        Model { settings, results}
    }

    pub fn get_banner(&self, testname: &String) -> String {
        let id = self.get_test_id_by_name(testname);
        self.settings.tests[id].banner.clone()
    }


    pub fn is_allowed_user(&self, username: &String, testname: &String) -> bool {
        let id = self.get_test_id_by_name(testname); 
        let test = &self.settings.tests[id]; 
        test.allowed_users.contains(username)
    }
   
    pub fn start_test(&mut self, username: &String, test: &String) -> Result<String, ()> {
        // auth
        if !self.is_user_done_test(username, test) {
            return Err(()); 
        }

        // TODO generate questions
        let variant = self.generate_variant(&test);
        self.create_test_record(username, test, variant);
        //self.get_next_question(&username, &test, None)
        Ok(self.get_banner(test))
    }
    
    fn generate_variant(&self, testname: &String) -> Variant {
        // TODO shuffle 
        let id = self.get_test_id_by_name(testname); 
        let test = &self.settings.tests[id];
        
        let mut questions: Vec<Question> = vec![];
        for i in 0..cmp::max(0, cmp::min(test.question_number, test.questions.len())) {
            questions.push(test.questions[i].clone()); 
        }

        Variant {questions, answers: vec![], current_question: 0, result: 0 }
    }

    fn get_test_id_by_name(&self, testname: &String) -> usize {
        for i in 0..self.settings.tests.len() {
            if &self.settings.tests[i].caption == testname {
                return i 
            }
        }
        0 // TODO
    }

    fn create_test_record(&mut self, username: &String, testname: &String, variant: Variant) {
        self.results.insert(username.to_owned() + "@" + testname, variant); 
    }


    pub fn get_avaliable_tests(&self, username: &String) -> Vec<(String, String)> {
        let mut res: Vec<(String, String)> = vec![];
        for test in &self.settings.tests {
            if test.allowed_users.contains(username) {
                res.push((test.caption.clone(), self.get_result(username, &test))) 
            }
        }
        /*for test in &self.settings.test {
            res.push(test.caption.clone());
        }*/
        res
    }

    /// Return [true] if user done the test.
    fn is_user_done_test(&self, _username: &String, _test: &String) -> bool {
        // TODO
        false 
    }

    pub fn get_next_question(&self, username: &String, testname: &String) -> Question {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            let id = self.results[&result_mark].current_question;
            if self.results[&result_mark].result == 0 {
                return self.results[&result_mark].questions[id].clone(); 
            }
        }
        
        // TODO
        Question {question: "111".to_string(), answers: ["tt".to_string()].to_vec(), correct_answers: [0].to_vec()}
    }

    pub fn is_next_question(&self, username: &String, testname: &String) -> bool {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            self.results[&result_mark].current_question < self.results[&result_mark].questions.len()
        } else {
           false 
        }
    }

    pub fn put_answer(&mut self, username: &String, testname: &String, answer: &Vec<u8>) {
        let result_mark = username.to_owned() + "@" + &testname;
        if self.results.contains_key(&result_mark) {
            self.results.get_mut(&result_mark).unwrap().answers.push(answer.clone()); 
            self.results.get_mut(&result_mark).unwrap().current_question += 1;
        }
    }


    fn get_result(&self, username: &String, test: &Test) -> String {
        let result_mark = username.to_owned() + "@" + &test.caption;
        if self.results.contains_key(&result_mark) {
            self.results[&result_mark].result.to_string()
        } else {
            "".to_string() 
        }
    }


    pub fn get_result_by_testname(&self, username: &String, testname: &String) -> String {
        let id = self.get_test_id_by_name(testname); 
        let test = &self.settings.tests[id]; 
        self.get_result(username, test)
    }

}


fn read_settings() -> Result<Settings, Box<dyn Error>> {
    let mut file = File::open("settings.toml")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}





