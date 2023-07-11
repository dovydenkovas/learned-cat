use std::io::{Read, Write};
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::cmp::Eq;

use serde_json::from_str as from_json;

mod network_structs;
use network_structs::{Request, Response, Command, NextQuestion};
mod parsetest;
mod model;
use parsetest::*;
use model::*;


fn handle_client(stream: &mut TcpStream, presenter: &mut Presenter) -> Result<(), Box<dyn Error>> {
    let mut request = [0 as u8; 5000];
    let n_bytes = stream.read(&mut request)?;
    let request = bincode::deserialize::<network_structs::Request>(&request[0..n_bytes])?;
    
    print!("[{}] {:?} -> ", chrono::Utc::now(),  request);
    let response = presenter.serve_connection(request);
    println!("{:?}", response);
    let response = bincode::serialize(&response)?;
    
    stream.write(&response)?;
    Ok(())
}


/// Open listener and run main loop
fn main() {
    let mut presenter: Presenter = Presenter::new();
    
    println!("* Открываю порт сервера: 127.0.0.1:65001");
    let listener = TcpListener::bind("127.0.0.1:65001").expect("Не могу открыть соединение");

    println!("* Запускаю главный цикл\n");
    loop {    
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    match handle_client(&mut stream, &mut presenter) {
                        Ok(()) => (),
                        Err(err) => eprintln!("{err:?}")
                    }   
                },
                Err(err) => eprintln!("{err:?}")
            }
        }
    }
}




struct Presenter { 
    model: Model
}

impl Presenter {
    pub fn new() -> Presenter {
        println!("Запуск сервера тестирования:");
        println!("* Чтение файла конфигурации ");
        let settings = read_settings().expect("Не могу прочитать файл конфигурации settings.json.");
        
        println!("* Чтение тестов: ");
        let quests = read_quests(&settings);
        for quest in &quests {
            println!("  * {}", quest.name);
        }

        Presenter { model: Model::new(settings, quests) }
    }

    pub fn serve_connection(&self, request: Request) -> Response {
        if self.is_allowed_user(&request.user) {
            match request.command {
                Command::GetAvaliableTests => {
                    return Response::AvaliableTests {
                        tests: self.model.get_avaliable_tests()
                    };
                },

                Command::StartTest { test } => {
                    self.start_test(&request.user, &test); 
                    let banner = self.model.get_banner(&test);
                    return Response::StartTest { banner: banner };
                },

     
                Command::GetNextQuestion { test, previos_answer } => {
                    return self.get_next_question(&request.user,
                                                  &test, 
                                                  &previos_answer);       
                }
            }
        } 
        Response::NotAllowedUser 
    }

    fn is_allowed_user(&self, username: &String) -> bool {
        true
    }

   

    fn start_test(&self, username: &String, test: &String) {
    
    }

    

    fn get_next_question(
        &self, 
        username: &String, 
        test: &String,
        previos_answer: &Vec<u8>) -> Response {
        Response::GetNextQuestion { 
            question: NextQuestion::TheEnd {
                result: "Молодец!".to_string()
            } 
        }
    }
}


fn read_settings() -> Result<Settings, Box<dyn Error>> {
    let mut file = File::open("settings.json")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_json(&settings)?)
}


fn read_quests(settings: &Settings) -> Vec<Quest> {
    let quests_base_path =  Path::new(&settings.quests_directory_path);
    let mut quests: Vec<Quest> = vec![];
    for test in &settings.quests_file_names {
        let path =  quests_base_path.join(Path::new(&test));
        quests.push(read_test(&path));
    }

    quests 
}


struct Model {
    database: Database,
    settings: Settings,
    quests: Vec<Quest>
}


impl Model {
    pub fn new(settings: Settings, quests: Vec<Quest> ) -> Model {
        Model { 
            database: Database::new(), 
            settings: settings,
            quests: quests
        }
    }

    fn get_banner(&self, test: &String) -> String {
        for quest in &self.quests {
            if quest.name.eq(test) {
                return quest.banner.clone()
            }
        }
        "".to_string()
    }

    pub fn init(&mut self) {
       self.read_configs();
       self.read_test_files();
       self.connect_database();
    }

    fn read_configs(&self) {
    
    }

    fn read_test_files(&self) {
    
    }

    /// Open database or create if not exist
    fn connect_database(&self) {
    }

    pub fn is_allowed_user(&self, username: &String) -> bool {
       true 
    }
    
    fn get_avaliable_tests(&self) -> Vec<String> {
        let mut res: Vec<String> = vec![];
        for quest in &self.quests {
            res.push(quest.name.clone());
        }
        res
    }
}


struct Test {
    name: String,
    banner: String,
    questions: Vec<Question>
}

struct Question {
    id: u32,
    question: String,
    answers: Vec<(u32, String)>
}


struct Database {
    //connection: sqlite::Connection 
}

impl Database {
    pub fn new() -> Database {
      //  let conn = sqlite::open("database.sqlite").unwrap();

        Database {
      //      connection: conn
        }
    }
}
