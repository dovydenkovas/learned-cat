use std::io::Write;

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
                    /*Err(ModelError::TestIsOpened(_)) => {
                        self.get_next_question(&request.user, &request.test)? 
                    },*/
                    Err(err) => return Err(err),
                }
            },

 
            Command::GetNextQuestion => {
                self.get_next_question(&request.user, &request.test)?
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
        let results = self.model.get_results();
        let mut file = std::fs::File::create(filename).expect("Не могу создать файл");
        
        for vars in results.values() {
            for var in &vars.variants {
                if var.result.is_some() {
                let out = format!("{},{},{},{},{}\n", 
                                  var.testname, 
                                  var.username, 
                                  var.timestamp.date_naive(), 
                                  var.timestamp.time().format("%H:%M:%S"), 
                                  var.result.unwrap());

                println!("{}", out);
                let _ = file.write(out.as_bytes());

                }
            }
        }
    }

    fn get_next_question(&mut self, user: &String, test: &String) -> ModelResult<Response> {
        if self.model.is_next_question(user, &test)? {
            let question = self.model.get_next_question(&user, &test)?;
            Ok(Response::NextQuestion {
                question: question.question, 
                answers: question.answers,
            })
        } else {
            Ok(Response::End { result: self.model.get_result_by_testname(&user, &test)? })
        }
    }
}
