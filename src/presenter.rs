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

    fn _serve_connection(&mut self, request: Request) -> ModelResult<Response> {
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

#[cfg(test)]
mod tests {
    use crate::model::{init, read_settings};

    use super::*;

    fn free_resource<S: AsRef<str>>(name: S) {
        let dir = std::env::temp_dir();
        let path = dir.join(name.as_ref());
        let _ = std::fs::remove_dir_all(&path);
    }

    fn get_test_presenter<S: AsRef<str>>(name: S) -> Presenter {
        let dir = std::env::temp_dir();
        let path = dir.join(name.as_ref());

        std::env::set_var("LEARNED_CAT_PATH", &path);
        init::init_server(path.as_path());
        let settings = read_settings().unwrap();
        Presenter::new(settings)
    }

    #[test]
    fn serve_connection_avaliable_test() {
        free_resource("test_scat");
        let mut presenter = get_test_presenter("test_scat");

        // not exists user
        let req = Request::new("cat_user", "", Command::GetAvaliableTests);

        match presenter.serve_connection(req) {
            Response::AvaliableTests { tests } => assert_eq!(tests, vec![]),
            _ => assert!(false),
        };

        // user with one test
        let req = Request::new("student1", "", Command::GetAvaliableTests);
        match presenter.serve_connection(req) {
            Response::AvaliableTests { mut tests } => {
                tests.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                assert_eq!(tests, vec![("linux".to_string(), "".to_string())])
            }
            _ => assert!(false),
        };

        // user with two tests
        let req = Request::new("student2", "", Command::GetAvaliableTests);
        match presenter.serve_connection(req) {
            Response::AvaliableTests { mut tests } => {
                tests.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                assert_eq!(
                    tests,
                    vec![
                        ("linux".to_string(), "".to_string()),
                        ("python".to_string(), "".to_string())
                    ]
                )
            }
            _ => assert!(false),
        };
        free_resource("test_scat");
    }

    #[test]
    fn serve_connection_start_test() {
        free_resource("test_scst");
        let mut presenter = get_test_presenter("test_scst");

        // not exist test
        let req = Request::new("student1", "fake_test", Command::StartTest);
        match presenter.serve_connection(req) {
            Response::NotAllowedUser => assert!(true),
            _ => assert!(false),
        }

        // not allowed user
        let req = Request::new("cat_user", "linux", Command::StartTest);
        match presenter.serve_connection(req) {
            Response::NotAllowedUser => assert!(true),
            _ => assert!(false),
        }
        /*
                // new user
                let req = Request::new("student1", "linux", Command::StartTest);
                match presenter.serve_connection(req) {
                    Response::TestStarted { banner } => assert!(banner.len() > 0),
                    resp => assert!(false, "got <{:?}> expected TestStarted", resp),
                }

                // test is running
                let req = Request::new("student1", "linux", Command::StartTest);
                presenter.serve_connection(req);
                let req = Request::new("student1", "linux", Command::GetNextQuestion);
                presenter.serve_connection(req);
                let req = Request::new("student1", "linux", Command::StartTest);
                match presenter.serve_connection(req) {
                    Response::NextQuestion { question, answers } => {
                        assert!(question.len() > 0 && answers.len() > 0)
                    }
                    _ => assert!(false),
                }

                // test is done
                let req = Request::new("student1", "linux", Command::PutAnswer { answer: vec![0] });
                presenter.serve_connection(req);
                let req = Request::new("student1", "linux", Command::StartTest);
                match presenter.serve_connection(req) {
                    Response::End { result } => assert!(true),
                    _ => assert!(false),
                }
        */
        free_resource("test_scst");
    }

    #[test]
    fn server_connection_getnext_test() {
        // not allowed user
        // new user
        // test is running
        // test is done
    }

    #[test]
    fn server_connection_putanswer_test() {
        // not allowed user
        // new user
        // test is running
        // test is done
    }
}
