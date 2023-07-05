#[macro_use] extern crate rocket;
use serde_json::to_string as to_json;
use serde_json::from_str as from_json;
use rocket::State;

mod network_structs;
use network_structs::{Request, Response, Command, NextQuestion};


#[launch]
fn rocket() -> _ {
    let presenter: Presenter = Presenter::new();
    rocket::build()
        .mount("/", routes![view])
        .manage(Presenter::new())
}


#[post("/", format = "json", data = "<request>")]
fn view(presenter: &State<Presenter>, request: &str) -> String {
    match from_json::<Request>(request) {
        Ok(request) => {
            let response = presenter.serve_connection(request);        
            to_json(&response).unwrap()
        },
        Err(_) => String::new(), // If request incorrect return empty string.
    }    
}



struct Presenter { 
    model: Model
}

impl Presenter {
    pub fn new() -> Presenter {
        Presenter { model: Model::new() }
    }

    pub fn serve_connection(&self, request: Request) -> Response {
        if self.is_allowed_user(&request.user) {
            match request.command {
                Command::GetAvaliableTests => {
                    return Response::AvaliableTests {
                        tests: self.get_avaliable_tests()
                    };
                },

                Command::StartTest { test } => {
                    self.start_test(&request.user, &test); 
                    let banner = self.get_banner(&test);
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

    fn get_avaliable_tests(&self) -> Vec<String> {
        vec!["calculate".to_string(), "alt".to_string()]
    }

    fn start_test(&self, username: &String, test: &String) {
    
    }

    fn get_banner(&self, test: &String) -> String {
        "".to_string()
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


struct Model {
    database: Database
}


impl Model {
    pub fn new() -> Model {
        Model { database: Database::new() }
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
