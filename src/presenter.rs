use crate::model::errors::{ModelError, ModelResult};
use crate::model::{Model, Settings};
use crate::network::{Command, Request, Response};
use std::sync::{Arc, Mutex};

pub struct Presenter {
    pub model: Arc<Mutex<Model>>,
}

impl Presenter {
    pub fn new(settings: Settings) -> Presenter {
        Presenter {
            model: Model::begin(settings),
        }
    }

    pub fn serve_connection(&mut self, request: Request) -> Response {
        match self._serve_connection(request) {
            Ok(response) => response,
            Err(err) => {
                eprintln!("{err}");
                Response::ServerError
            }
        }
    }

    pub fn _serve_connection(&mut self, request: Request) -> ModelResult<Response> {
        if request.command == Command::GetAvaliableTests {
            return Ok(Response::AvaliableTests {
                tests: self
                    .model
                    .lock()
                    .unwrap()
                    .get_avaliable_tests(&request.user)?,
            });
        }

        if !self
            .model
            .lock()
            .unwrap()
            .is_allowed_user(&request.user, &request.test)?
        {
            return Ok(Response::NotAllowedUser);
        }

        let response = match request.command {
            Command::StartTest => {
                match self
                    .model
                    .lock()
                    .unwrap()
                    .start_test(&request.user, &request.test)
                {
                    Ok(banner) => Response::TestStarted { banner },
                    Err(ModelError::TestIsDone) => Response::End {
                        result: self
                            .model
                            .lock()
                            .unwrap()
                            .get_result_by_testname(&request.user, &request.test)?,
                    },
                    /*Err(ModelError::TestIsOpened(_)) => {
                        self.get_next_question(&request.user, &request.test)?
                    },*/
                    Err(err) => return Err(err),
                }
            }

            Command::GetNextQuestion => self.get_next_question(&request.user, &request.test)?,

            Command::PutAnswer { answer } => {
                self.model
                    .lock()
                    .unwrap()
                    .put_answer(&request.user, &request.test, &answer)?;
                if self
                    .model
                    .lock()
                    .unwrap()
                    .is_next_question(&request.user, &request.test)?
                {
                    Response::Ok
                } else {
                    Response::End {
                        result: self
                            .model
                            .lock()
                            .unwrap()
                            .get_result_by_testname(&request.user, &request.test)?,
                    }
                }
            }

            Command::GetAvaliableTests => return Err(ModelError::UserNotAllowed),
        };

        Ok(response)
    }

    fn get_next_question(&mut self, user: &String, test: &String) -> ModelResult<Response> {
        if self.model.lock().unwrap().is_next_question(user, &test)? {
            let question = self.model.lock().unwrap().get_next_question(&user, &test)?;
            Ok(Response::NextQuestion {
                question: question.question,
                answers: question.answers,
            })
        } else {
            Ok(Response::End {
                result: self
                    .model
                    .lock()
                    .unwrap()
                    .get_result_by_testname(&user, &test)?,
            })
        }
    }
}
