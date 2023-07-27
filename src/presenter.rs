use crate::model::network::{Request, Response, Command};
use crate::model::Model;

pub struct Presenter { 
    pub model: Model
}

impl Presenter {
    pub fn new() -> Presenter {
        Presenter { model: Model::new() }
    }

    pub fn serve_connection(&self, request: Request) -> Response {
        if self.model.is_allowed_user(&request.user) {
            match request.command {
                Command::GetAvaliableTests => {
                    return Response::AvaliableTests {
                        tests: self.model.get_avaliable_tests()
                    };
                },

                Command::StartTest { test } => {
                    self.model.start_test(&request.user, &test); 
                    let banner = self.model.get_banner(&test);
                    return Response::StartTest { banner: banner };
                },

     
                Command::GetNextQuestion { test, previos_answer } => {
                    return self.model.get_next_question(&request.user,
                                                  &test, 
                                                  Some(&previos_answer));       
                }
            }
        } 
        Response::NotAllowedUser 
    }
    

    
    pub fn export_results(&mut self, filename: String) {
        self.model.get_results();   

        println!("Созраняю в файл {filename}.");
        // TODO: !!
    }
}

