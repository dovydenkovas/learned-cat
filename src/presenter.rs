use crate::network::{Request, Response, Command};
use crate::model::Model;

pub struct Presenter { 
    pub model: Model
}

impl Presenter {
    pub fn new() -> Presenter {
        Presenter { model: Model::new() }
    }

    pub fn serve_connection(&mut self, request: Request) -> Response {
        if !self.model.is_allowed_user(&request.user, &request.test) {
            return Response::NotAllowedUser;
        }
        
        // Better error check 
        match request.command {
            Command::GetAvaliableTests => {
                Response::AvaliableTests {
                    tests: self.model.get_avaliable_tests(&request.user),
                }
            },

            Command::StartTest => {
                match self.model.start_test(&request.user, &request.test) {
                    Ok(banner) =>  Response::TestStarted { banner },
                    Err(_) => Response::End{ result: self.model.get_result_by_testname(&request.user, &request.test) } 
                }
            },

 
            Command::GetNextQuestion => {
                if self.model.is_next_question(&request.user, &request.test) {
                    let question = self.model.get_next_question(&request.user, &request.test);
                    Response::NextQuestion {
                        question: question.question, 
                        answers: question.answers,
                    }
                } else {
                    Response::End { result: self.model.get_result_by_testname(&request.user, &request.test) }
                }
            },

            Command::PutAnswer { answer } => {
                self.model.put_answer(&request.user, &request.test, &answer);
                if self.model.is_next_question(&request.user, &request.test) {
                    Response::Ok 
                } else {
                    Response::End { result: self.model.get_result_by_testname(&request.user, &request.test) }
                }
            },

        }
    }
    

    pub fn export_results(&mut self, filename: String) {
        //self.model.get_results();   

        println!("Созраняю в файл {filename}.");
        // TODO: export results
    }
}
