#![allow(dead_code)]

/// Содержит структуры тестов
use std::io::Read;
use std::fs::File;
use std::error::Error;
use std::path::Path;

use toml::from_str as from_toml;
use postgres::{Client as Database, NoTls};
use serde::Deserialize;


pub mod network;
pub mod parsetest;
pub mod init;
use crate::model::parsetest::read_test;



#[derive(Deserialize, Debug)]
struct Settings {
    tests_directory_path: Option<String>,
    allowed_table_path: Option<Vec<String>>,
    allowed_table: Option<Vec<String>>,
    test: Option<Vec<Test>>
}


#[derive(Debug, Deserialize)]
pub struct Test {
    /// Basic info
    pub caption: String,
    pub banner: Option<String>,
    
    /// Variant parameters
    pub questions: Option<Vec<Question>>,
    pub question_number: Option<u16>,
    pub shuffle: Option<bool>,
    pub test_duration: Option<u16>,
    
    /// Castumization 
    pub show_results: Option<bool>
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
    database: Database,
    settings: Settings,
}


impl Model {
    pub fn new() -> Model {
        println!("* Чтение файла конфигурации ");
        let mut settings = read_settings().expect("Не могу прочитать файл конфигурации settings.json.");
        println!("* Чтение тестов: ");
       
        // Read tests
        let quests_base_path =  Path::new(&settings.tests_directory_path.unwrap_or("tests".to_string()));
        match &mut settings.test {
            Some(tests) => {
                for test in tests {
                    let path =  quests_base_path.join(Path::new(&(test.caption)));
                    read_test(&path, test);
                }
            }

            None => eprintln!("Нет ни одного теста")
        };

                
        for test in &settings.test.unwrap() {
            println!("  * {}", test.caption);
        }
        
        let database = match 
            Database::connect("host=localhost user=asd dbname=sshtest", NoTls) {
            Ok(db) => db,
            Err(err) => { 
                eprintln!("{err:?}");
                std::process::exit(1);
            }
            };

        Model {database, settings}
    }

    pub fn get_banner(&self, test: &String) -> String {
        for quest in &self.settings.test.unwrap_or(vec![]) {
            if quest.caption.eq(test) {
                return quest.banner.unwrap().clone()
            }
        }
        "".to_string()
    }

    

    pub fn is_allowed_user(&self, username: &String) -> bool {
        self.settings.allowed_users.contains(username) 
    }
   
    pub fn start_test(&self, username: &String, test: &String) -> network::Response {
        // auth
        if !self.is_user_done_test(username, test) {
            return network::Response::NotAllowedUser; 
        }

        // TODO generate questions
        let variant: Vec<u16> = self.generate_variant(&test);
        self.create_test_record(username, test, variant);
        self.get_next_question(&username, &test, None)
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


    pub fn get_avaliable_tests(&self) -> Vec<String> {
        let mut res: Vec<String> = vec![];
        for test in &self.settings.test {
            res.push(test.caption.clone());
        }
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
        previos_answer: Option<&Vec<u8>>) -> network::Response {
        
        if self.is_user_done_test(username, test) {
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
        }
    }

    // Open database or create if not exist
    //fn connect_database(&mut self) -> Database {
    //    self.database.batch_execute("").unwrap();
//        insert into student (login, is_allowed) VALUES ('student-1', true);
// insert into test (caption) VALUES ('Python_test');
    //}

    
}


fn read_settings() -> Result<Settings, Box<dyn Error>> {
    let mut file = File::open("settings.toml")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_toml(&settings)?)
}





