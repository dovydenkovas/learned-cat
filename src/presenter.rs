use crate::network::{Request, Response, Command};
use crate::model::Model;
use crate::model::errors::{ModelResult, ModelError};


pub struct Presenter { 
    pub model: Model
}

impl Presenter {
    pub fn new() -> Presenter {
        Presenter { model: Model::new() }
    }

    pub fn serve_connection(&mut self, request: Request) -> Response {
        match self._serve_connection(request) {
            Ok(response) => response,
            Err(err) => {
                eprintln!("{err}");
                Response::ServerError
            },
        }
    }

    pub fn _serve_connection(&mut self, request: Request) -> ModelResult<Response> {
        if request.command == Command::GetAvaliableTests {
            return Ok(Response::AvaliableTests {
                    tests: self.model.get_avaliable_tests(&request.user)?,
            });
        }
    

        if !self.model.is_allowed_user(&request.user, &request.test)? {
            return Ok(Response::NotAllowedUser);
        }
        
        let response = match request.command {
            Command::StartTest => {
                match self.model.start_test(&request.user, &request.test) {
                    Ok(banner) => Response::TestStarted { banner },
                    Err(ModelError::TestIsDone) => Response::End { 
                        result: self.model.get_result_by_testname(&request.user, &request.test)? 
                    },
                    Err(err) => return Err(err),
                }
            },

 
            Command::GetNextQuestion => {
                if self.model.is_next_question(&request.user, &request.test)? {
                    let question = self.model.get_next_question(&request.user, &request.test)?;
                    Response::NextQuestion {
                        question: question.question, 
                        answers: question.answers,
                    }
                } else {
                    Response::End { result: self.model.get_result_by_testname(&request.user, &request.test)? }
                }
            },

            Command::PutAnswer { answer } => {
                self.model.put_answer(&request.user, &request.test, &answer)?;
                if self.model.is_next_question(&request.user, &request.test)? {
                    Response::Ok 
                } else {
                    Response::End { result: self.model.get_result_by_testname(&request.user, &request.test)? }
                }
            },

            Command::GetAvaliableTests => return Err(ModelError::UserNotAllowed)

        };

        Ok(response)
    }
    

    pub fn export_results(&mut self, filename: String) {
        //self.model.get_results();   

        println!("Созраняю в файл {filename}.");
        // TODO: export results
    }
}
