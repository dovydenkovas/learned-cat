use crate::model::errors::{ModelError, ModelResult};
use crate::model::{Model, Settings};
use crate::network::{Command, Request, Response};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Presenter {
    pub model: Arc<Mutex<Model>>,
}

impl Presenter {
    pub fn new<P: AsRef<Path>>(settings: Settings, root_path: P) -> Presenter {
        Presenter {
            model: Model::begin(settings, root_path.as_ref()),
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
            Command::GetNextQuestion => {
                let res = self
                    .model
                    .lock()
                    .unwrap()
                    .start_test(&request.user, &request.test);
                match res {
                    Ok(banner) => Response::TestStarted { banner },
                    Err(ModelError::TestIsDone) => Response::End {
                        result: self
                            .model
                            .lock()
                            .unwrap()
                            .get_result_by_testname(&request.user, &request.test)?,
                    },
                    Err(ModelError::TestIsOpened(_, _)) => {
                        self.get_next_question(&request.user, &request.test)?
                    }
                    Err(err) => return Err(err),
                }
            }

            //Command::GetNextQuestion => match self.get_next_question(&request.user, &request.test)?,
            Command::PutAnswer { answer } => {
                let res =
                    self.model
                        .lock()
                        .unwrap()
                        .put_answer(&request.user, &request.test, &answer);
                match res {
                    Ok(_) => (),
                    Err(ModelError::TestIsDone) => {
                        return Ok(Response::End {
                            result: (self
                                .model
                                .lock()
                                .unwrap()
                                .get_result_by_testname(&request.user, &request.test)?),
                        })
                    }
                    Err(err) => return Err(err),
                };
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

        init::init_server(path.as_path());
        let settings = read_settings(path.as_path()).unwrap();
        Presenter::new(settings, path.as_path())
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
        let req = Request::new("student1", "fake_test", Command::GetNextQuestion);
        assert_eq!(presenter.serve_connection(req), Response::NotAllowedUser);

        // not allowed user
        let req = Request::new("cat_user", "linux", Command::GetNextQuestion);
        assert_eq!(presenter.serve_connection(req), Response::NotAllowedUser);

        // new user
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        assert!(matches!(
            presenter.serve_connection(req),
            Response::TestStarted { .. }
        ));

        // test is running
        let req = Request::new("student1", "linux", Command::GetNextQuestion);

        presenter.serve_connection(req);
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        presenter.serve_connection(req);
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        assert!(matches!(
            presenter.serve_connection(req),
            Response::NextQuestion { .. }
        ));

        // test is done
        let req = Request::new("student1", "linux", Command::PutAnswer { answer: vec![0] });
        presenter.serve_connection(req);
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        assert_eq!(
            presenter.serve_connection(req),
            Response::End {
                result: "0".to_string()
            }
        );

        free_resource("test_scst");
    }

    #[test]
    fn server_connection_getnext_test() {
        free_resource("test_scgt");
        let mut presenter = get_test_presenter("test_scgt");

        // not allowed user
        let req = Request::new("cat_user", "linux", Command::GetNextQuestion);
        assert_eq!(presenter.serve_connection(req), Response::NotAllowedUser);

        // new user
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        assert!(matches!(
            presenter.serve_connection(req),
            Response::TestStarted { .. }
        ));

        // test is running

        // test is done

        free_resource("test_scgt");
    }

    #[test]
    fn server_connection_putanswer_test() {
        free_resource("test_scpt");
        let mut presenter = get_test_presenter("test_scpt");

        // not allowed user
        let req = Request::new("cat_user", "linux", Command::PutAnswer { answer: vec![0] });
        assert_eq!(presenter.serve_connection(req), Response::NotAllowedUser);

        // new user
        let req = Request::new("student1", "linux", Command::PutAnswer { answer: vec![0] });
        assert_eq!(presenter.serve_connection(req), Response::ServerError);

        // test is started
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        presenter.serve_connection(req);
        let req = Request::new("student1", "linux", Command::PutAnswer { answer: vec![0] });
        assert_eq!(presenter.serve_connection(req), Response::ServerError);

        // test is running
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        presenter.serve_connection(req);
        let req = Request::new("student1", "linux", Command::GetNextQuestion);
        presenter.serve_connection(req);
        let req = Request::new("student1", "linux", Command::PutAnswer { answer: vec![0] });
        assert!(matches!(
            presenter.serve_connection(req),
            Response::End { .. }
        ));

        // test is done
        let req = Request::new("student1", "linux", Command::PutAnswer { answer: vec![0] });
        assert!(matches!(
            presenter.serve_connection(req),
            Response::End { .. }
        ));

        free_resource("test_scpt");
    }
}
