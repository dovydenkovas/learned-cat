/// Содержит структуры тестов
use std::io::Read;
use std::fs::File;
use std::error::Error;
use std::path::Path;

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
    pub question_number: u16,
    pub shuffle: bool,
    pub test_duration: u16,
    
    /// Castumization 
    pub show_results: bool
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
            show_results: true
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct Question {
    pub question: String, 
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>
}


#[derive(Debug)]
struct User {
    username: String,
    allowed_tests: Vec<String>,
}


pub struct Model {
    settings: Settings,
    results: std::collections::hash_map::HashMap<String, u16> 
}


impl Model {
    pub fn new() -> Model {
        println!("* Чтение файла конфигурации ");
        let mut settings = read_settings().expect("Не могу прочитать файл конфигурации settings.json.");
        println!("* Чтение тестов: ");
       
        // Read tests
        let quests_base_path = Path::new("tests"); // Path::new(&settings.tests_directory_path.unwrap_or("tests".to_string()));
        for test in &mut settings.tests {
            let path =  quests_base_path.join(Path::new(&(test.caption)));
            read_test(&path, test);
        }
                
        /*for test in settings.test.unwrap().iter() {
            println!("  * {}", test.caption);
        }*/

        Model { 
            settings, 
            results: std::collections::hash_map::HashMap::new()
        }
    }

    pub fn get_banner(&self, test: &String) -> String {
        /*for quest in &self.settings.test.unwrap_or(vec![]) {
            if quest.caption.eq(test) {
                return quest.banner.unwrap().clone()
            }
        }*/
        "".to_string()
    }

    

    pub fn is_allowed_user(&self, username: &String, test: &String) -> bool {
        //self.settings.allowed_users.contains(username) 
        true
    }
   
    pub fn start_test(&self, username: &String, test: &String) -> Result<String, ()> {
        // auth
        if !self.is_user_done_test(username, test) {
            return Err(()); 
        }

        // TODO generate questions
        let variant: Vec<u16> = self.generate_variant(&test);
        self.create_test_record(username, test, variant);
        //self.get_next_question(&username, &test, None)
        Ok(self.get_banner(test))
    }
    
    fn generate_variant(&self, test: &String) -> Vec<u16> {
        vec![]     
    }

    fn create_test_record(&self, username: &String, test: &String, variant: Vec<u16>) {
        /*for row in client.query("SELECT id, name, data FROM person", &[])? {
            let id: i32 = row.get(0);
            let name: &str = row.get(1);
            let data: Option<&[u8]> = row.get(2);

            println!("found person: {} {} {:?}", id, name, data);
        }*/


        /*let name = "Ferris";
        let data = None::<&[u8]>;
        client.execute(
            "INSERT INTO person (name, data) VALUES ($1, $2)",
            &[&name, &data],
        )?;*/

    }


    pub fn get_avaliable_tests(&self, username: &String) -> Vec<(String, String)> {
        let mut res: Vec<(String, String)> = vec![];
        /*for test in &self.settings.test {
            res.push(test.caption.clone());
        }*/
        res
    }

    pub fn get_results(&self) -> Vec<String> {
        // TODO
        vec![] 
    }
    
    /// Return [true] if user done the test.
    fn is_user_done_test(&self, _username: &String, _test: &String) -> bool {
        // TODO
        false 
    }

    pub fn get_next_question(
        &self, 
        username: &String, 
        test: &String,
        ) -> Question {
        
        /*if self.is_user_done_test(username, test) {
            network::Response::GetNextQuestion { 
                question: network::NextQuestion::TheEnd {
                    result: "Молодец!".to_string()
                } 
            }
        } else {
            network::Response::GetNextQuestion {
                question: network::NextQuestion::Question {
                    question: "2 + 2 = ?".to_string(),
                    answers: vec!["1".to_string(), "4".to_string(), "3".to_string()]
                }
            }
        }*/

        Question {question: "111".to_string(), answers: ["tt".to_string()].to_vec(), correct_answers: [0].to_vec()}
    }

    pub fn is_next_question(&self, username: &String, test: &String) -> bool {
        true
    }

    pub fn put_answer(&self, username: &String, test: &String, answer: &Vec<u8>) {
    
    }

    pub fn get_result(&self, username: &String, test: &String) -> String {
        "aaa".to_string()
    }
}


fn read_settings() -> Result<Settings, Box<dyn Error>> {
    let mut file = File::open("settings.toml")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}





