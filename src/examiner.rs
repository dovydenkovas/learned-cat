use std::collections::HashMap;

use learned_cat_interfaces::network::Request;
use log::{debug, error};
use rand::seq::SliceRandom;
use rand::thread_rng;

use learned_cat_interfaces::schema::{Answer, Variant};
use learned_cat_interfaces::{
    network::{Command, Response},
    schema::Question,
};
use learned_cat_interfaces::{Config, Database};

use crate::{ExaminerChannel, Tick};

pub struct Examiner {
    config: Box<dyn Config>,
    db: Box<dyn Database>,
    channel: ExaminerChannel,
    /// Хранилище вариантов - username - variants
    variants: HashMap<String, Variant>,
}

impl Examiner {
    pub fn new(
        config: Box<dyn Config>,
        db: Box<dyn Database>,
        channel: ExaminerChannel,
    ) -> Examiner {
        let examiner = Examiner {
            config,
            db,
            channel,
            variants: HashMap::new(),
        };
        examiner
    }

    /// Главный цикл обработки запросов.
    pub fn mainloop(&mut self) {
        loop {
            match self.channel.rx.recv() {
                Ok(tick) => match tick {
                    Tick::CollectCompletedTests => self.variant_collector_save(),
                    Tick::Request { request } => {
                        let responce = self.serve_request(request);
                        self.channel.tx.send(responce).unwrap();
                    }
                },
                Err(err) => error!("Ошибка обработки запроса: {:?}.", err),
            }
        }
    }

    /// Обработать запрос клиента.
    fn serve_request(&mut self, request: Request) -> Response {
        match request.command {
            Command::StartTest => self.banner_to_start_test_saved(&request.user, &request.test),
            Command::GetNextQuestion => self.next_question_saved(&request.user, &request.test),
            Command::GetAvaliableTests => self.avaliable_tests_saved(&request.user),
            Command::PutAnswer { answer } => {
                self.put_answer_saved(&request.user, &request.test, &answer)
            }
        }
    }

    /// Показать описание теста перед запуском
    fn banner_to_start_test_saved(&mut self, username: &String, testname: &String) -> Response {
        // У пользователя может не быть доступа.
        if !self.config.has_access(username, testname) {
            debug!(
                "Пользователь {username} пытался запустить тест {testname} не имея не это прав."
            );
            return Response::NotAllowedUser;
        }

        // Или закончатся попытки.
        if !self.has_attempt(username, testname) {
            debug!(
                "У пользователя {username} больше не осталось попыток на прохождение {testname}."
            );
            return Response::End {
                result: self.db.marks(username, testname),
            };
        }

        // Отправить описание теста.
        Response::TestStarted {
            banner: self.config.test_banner(testname).unwrap_or("".to_string()),
        }
    }

    /// Предоставить список доступных тестов.
    fn avaliable_tests_saved(&mut self, username: &String) -> Response {
        if !self.config.has_user(username) {
            error!("Пользователя {username} не существует.");
            Response::NotAllowedUser
        } else {
            let user_tests = self.config.user_tests_list(username);
            let mut tests = vec![];
            for test in &user_tests {
                tests.push((test.clone(), self.db.marks(username, test)));
            }
            Response::AvaliableTests { tests }
        }
    }

    /// Сохранить ответ на вопрос, отправить следующий вопрос или оценку.
    fn put_answer_saved(
        &mut self,
        username: &String,
        testname: &String,
        answer: &Answer,
    ) -> Response {
        // У пользователя может не быть доступа.
        if !self.config.has_access(username, testname) {
            error!(
                "Пользователь {username} пытался ответить на вопрос теста {testname} не имея не это прав."
            );
            return Response::NotAllowedUser;
        }
        // Тест может быть не запущен
        if !self.is_user_have_opened_variant(username, testname) {
            error!("Тест завершен, нельзя отвечать на вопросы: {username}, {testname} {answer:?}");
            return Response::End {
                result: self.db.marks(username, testname),
            };
        }
        self.push_answer_on_current_question(username, &answer);
        self.next_question_saved(username, testname)
    }

    /// Запустить тест или отправить новый вопрос.
    fn next_question_saved(&mut self, username: &String, testname: &String) -> Response {
        // У пользователя может не быть доступа.
        if !self.config.has_access(username, testname) {
            error!(
                "Пользователь {username} пытался получить следующий вопрос {testname} не имея доступа к тесту."
            );
            return Response::NotAllowedUser;
        }

        // Или закончатся попытки.
        if !self.has_attempt(username, testname) {
            debug!(
                "У пользователя {username} больше не осталось попыток на прохождение {testname}."
            );
            return Response::End {
                result: self.db.marks(username, testname),
            };
        }

        // Если пользователь ещё не начал тестирование.
        if !self.is_user_have_opened_variant(username, testname) {
            self.start_test(username, testname);
        }

        // Если есть неотвеченные вопросы.
        if self.is_next_question(username) {
            self.get_next_question(username)
        } else {
            self.done_test(username, testname);
            Response::End {
                result: self.db.marks(username, testname),
            }
        }
    }

    /// Проверка наличия попыток у пользователя.
    fn has_attempt(&mut self, username: &String, testname: &String) -> bool {
        let number_of_attempts = match self.config.test_settings(testname) {
            Some(conf) => conf.number_of_attempts,
            None => {
                error!("Тест {testname} требуемый пользователем {username} не обнаружен");
                0
            }
        };

        number_of_attempts <= 0 || self.db.attempts_counter(username, testname) < number_of_attempts
    }

    /// Есть ли у пользователя незаконченный тест.
    fn is_user_have_opened_variant(&self, username: &String, testname: &String) -> bool {
        self.variants.contains_key(username) && &self.variants[username].testname == testname
    }

    /// Возвращает первый неотвеченный вопрос.
    fn get_next_question(&mut self, username: &String) -> Response {
        let variant = &self.variants[username];
        let id = variant.answers.len();
        let question = variant.questions[id].clone();
        Response::NextQuestion {
            question: question.question,
            answers: question.answers,
        }
    }

    /// Запускает новый тест.
    fn start_test(&mut self, username: &String, testname: &String) {
        let variant = self.generate_variant(username, testname);
        self.create_test_record(username, variant);
        debug!("Пользователь {username} начал тестирование {testname}.");
    }

    /// Создать вариант теста.
    fn generate_variant(&self, username: &String, testname: &String) -> Variant {
        let test_settings = self.config.test_settings(testname).unwrap();

        let mut vec: Vec<usize> = (0..self.config.questions_count(testname).unwrap()).collect();
        vec.shuffle(&mut thread_rng());

        let mut questions: Vec<Question> = vec![];
        for i in 0..test_settings.questions_number {
            questions.push(self.config.question(testname, vec[i]).unwrap().clone());
        }

        Variant {
            username: username.clone(),
            testname: testname.clone(),
            start_timestamp: chrono::offset::Local::now(),
            questions,
            answers: vec![],
        }
    }

    /// Запомнить сгенерированный вариант теста.
    fn create_test_record(&mut self, username: &String, variant: Variant) {
        self.variants.insert(username.clone(), variant);
    }

    /// Закончилось ли время тестирования?
    fn is_test_time_is_over(&self, username: &String, testname: &String) -> bool {
        let test_settings = self.config.test_settings(testname).unwrap();
        let variant = &self.variants[username];

        chrono::Local::now() - variant.start_timestamp
            > chrono::Duration::new(test_settings.test_duration_minutes * 60, 0).unwrap()
    }

    /// Содержит ли вариант ещё неотвеченные вопросы.
    fn is_next_question(&self, username: &String) -> bool {
        let current_question = self.variants[username].answers.len();
        return current_question < self.variants[username].questions.len();
    }

    /// Сохранить ответ пользователя на последний неотвеченный вопрос.
    fn push_answer_on_current_question(&mut self, username: &String, answer: &Answer) {
        let variant = self.variants.get_mut(username).unwrap();
        variant.answers.push(answer.clone());
    }

    /// Завершить тест
    fn done_test(&mut self, username: &String, testname: &String) {
        let mark = self.calculate_mark(username);
        let start_time = self.variants[username].start_timestamp.to_string();
        let end_time = chrono::Local::now().to_string();
        self.db
            .append_mark(username, testname, mark, &start_time, &end_time);
        debug!(
            "Пользователь {username} завершил тест {testname}: {:?}.",
            self.variants[username]
        );
        self.variants.remove(username);
    }

    /// Посчитать оценку за тест.
    fn calculate_mark(&mut self, username: &String) -> f32 {
        let variant = self.variants.get_mut(username).unwrap();
        let mut result: f32 = 0.0;
        for i in 0..variant.answers.len() {
            if variant.questions[i].correct_answer == variant.answers[i] {
                result += 1.0;
            }
        }
        result
    }

    fn variant_collector_save(&mut self) {
        let mut done_tests = vec![];

        for it in &self.variants {
            let variant = it.1;
            if self.is_test_time_is_over(&variant.username, &variant.testname) {
                done_tests.push((variant.username.clone(), variant.testname.clone()));
            }
        }

        for variant in done_tests {
            self.done_test(&variant.0, &variant.1);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use learned_cat_interfaces::{
//         network,
//         schema::{Answer, Question},
//         settings::{self},
//         Config, Database, Server,
//     };

//     use super::Examiner;

//     struct TDatabase {}

//     #[allow(unused)]
//     impl Database for TDatabase {
//         fn attempts_counter(&mut self, username: &String, testname: &String) -> u32 {
//             2
//         }

//         fn marks(&mut self, username: &String, testname: &String) -> Vec<f32> {
//             vec![3.0, 4.0, 5.0]
//         }

//         /// Сохранить баллы за тест testname для пользователя username.
//         fn append_mark(
//             &mut self,
//             username: &String,
//             testname: &String,
//             mark: f32,
//             start_timestamp: &String,
//             end_timestamp: &String,
//         ) {
//         }
//     }

//     struct TServer {}

//     impl Server for TServer {
//         fn pop_request(&mut self) -> Option<network::Request> {
//             Some(network::Request::new(
//                 "user",
//                 "test",
//                 network::Command::GetNextQuestion,
//             ))
//         }

//         fn push_response(&mut self, response: network::Response) {
//             assert_ne!(response, network::Response::ResponseError);
//         }
//     }

//     struct TConfig {}
//     #[allow(unused)]
//     impl Config for TConfig {
//         fn has_user(&self, username: &String) -> bool {
//             true
//         }

//         fn has_test(&self, testname: &String) -> bool {
//             true
//         }

//         fn test_settings(&self, testname: &String) -> Option<settings::TestSettings> {
//             Some(settings::TestSettings {
//                 caption: "math".to_string(),
//                 questions_number: 2,
//                 test_duration_minutes: 1,
//                 number_of_attempts: 3,
//                 show_results: true,
//                 allowed_users: vec!["user".to_string()],
//             })
//         }

//         fn test_banner(&self, testname: &String) -> Option<String> {
//             Some("description".to_string())
//         }

//         fn question(&self, testname: &String, question_id: usize) -> Option<Question> {
//             Some(Question {
//                 question: "text".to_string(),
//                 answers: vec!["A".to_string(), "B".to_string()],
//                 correct_answer: Answer::new(vec![1, 2, 3]),
//             })
//         }

//         /// Получить количество вопросов в тесте.
//         fn questions_count(&self, testname: &String) -> Option<usize> {
//             Some(1)
//         }

//         fn answer(&self, testname: &String, question_id: usize) -> Option<Answer> {
//             Some(Answer::new(vec![1, 2]))
//         }

//         fn has_access(&self, username: &String, testname: &String) -> bool {
//             true
//         }

//         fn user_tests_list(&self, username: &String) -> Vec<String> {
//             vec!["A".to_string(), "B".to_string(), "C".to_string()]
//         }

//         fn settings(&self) -> settings::Settings {
//             settings::Settings {
//                 tests_directory_path: "example-config".to_string(),
//                 result_path: "marks.db".to_string(),
//                 server_address: "127.0.0.1:8080".to_string(),
//                 tests: vec![self.test_settings(&"math".to_string()).unwrap()],
//                 new_file_permissions: 0x660,
//             }
//         }
//     }
//     // #[test]
//     // fn examiner() {
//     //     let config = TConfig {};
//     //     let database = TDatabase {};
//     //     let server = TServer {};
//     //     let mut examiner = Examiner::new(Box::new(config), Box::new(database), Box::new(server));
//     //     //examiner.mainloop();
//     //     //assert!(false);
//     // }
// }
