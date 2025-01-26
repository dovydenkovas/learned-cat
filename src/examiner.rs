use std::collections::HashMap;

use learned_cat_interfaces::network::Request;
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
            let tick = self.channel.rx.recv().unwrap();
            match tick {
                Tick::CollectCompletedTests => self.variant_collector(),
                Tick::Request { request } => {
                    let responce = self.serve_request(request);
                    self.channel.tx.send(responce).unwrap();
                }
            }
        }
    }

    fn serve_request(&mut self, request: Request) -> Response {
        match request.command {
            Command::StartTest | Command::GetNextQuestion => {
                self.next_question(&request.user, &request.test)
            }

            Command::GetAvaliableTests => self.avaliable_tests(&request.user),

            Command::PutAnswer { answer } => self.put_answer(&request.user, &request.test, &answer),
        }
    }

    /// Предоставить список доступных тестов.
    fn avaliable_tests(&self, username: &String) -> Response {
        if !self.config.has_user(username) {
            Response::NotAllowedUser
        } else {
            Response::AvaliableTests {
                tests: self.config.user_tests_list(username),
            }
        }
    }

    /// Сохранить ответ на вопрос.
    fn put_answer(&mut self, username: &String, testname: &String, answer: &Answer) -> Response {
        self._put_answer(username, testname, &answer);
        if self.is_next_question(username, testname) {
            Response::Ok
        } else {
            Response::End {
                result: self.db.marks(username, testname),
            }
        }
    }

    /// Запустить тест или отправить новый вопрос.
    fn next_question(&mut self, username: &String, testname: &String) -> Response {
        if !self.config.has_access(username, testname) {
            return Response::NotAllowedUser;
        }

        if self.db.attempts_counter(username, testname)
            < self
                .config
                .test_settings(testname)
                .unwrap()
                .number_of_attempts
        {
            return Response::End {
                result: self.db.marks(username, testname),
            };
        }

        if self.is_user_have_opened_variant(username, testname) {
            if self.is_test_time_is_over(username, testname) {
                self.done_test(username, testname);
                return Response::End {
                    result: self.db.marks(username, testname),
                };
            } else {
                return self.get_next_question(username);
            }
        }

        self.start_test(username, testname);
        Response::TestStarted {
            banner: self.config.test_banner(testname).unwrap(),
        }
    }

    /// Возвращает первый неотвеченный вопрос.
    fn get_next_question(&mut self, username: &String) -> Response {
        let variant = self.variants.get_mut(username).unwrap();
        if variant.start_timestamp.is_none() {
            variant.start_timestamp = Some(chrono::offset::Local::now());
        }

        let mut id = variant.current_question;
        if variant.current_question.is_none() {
            variant.current_question = Some(0);
            id = Some(0);
        }

        let question = variant.questions[id.unwrap()].clone();
        Response::NextQuestion {
            question: question.question,
            answers: question.answers,
        }
    }

    /// Запускает новый тест.
    fn start_test(&mut self, username: &String, testname: &String) {
        let variant = self.generate_variant(username, testname);
        self.create_test_record(username, variant);
    }

    /// Есть ли у пользователя незаконченный тест.
    fn is_user_have_opened_variant(&self, username: &String, testname: &String) -> bool {
        self.variants.contains_key(username) && &self.variants[username].testname == testname
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
            start_timestamp: None,
            questions,
            answers: vec![],
            current_question: None,
        }
    }

    /// Запомнить сгенерированный вариант теста.
    fn create_test_record(&mut self, username: &String, variant: Variant) {
        if self.variants.contains_key(username) {
            eprintln!("ERROR: User already has opened variant. Skip.");
        } else {
            self.variants.insert(username.clone(), variant);
        }
    }

    /// Закончилось ли время тестирования?
    fn is_test_time_is_over(&self, username: &String, testname: &String) -> bool {
        if !self.is_user_have_opened_variant(username, testname) {
            return false;
        }
        let test_settings = self.config.test_settings(testname).unwrap();
        let variant = &self.variants[username];
        if variant.start_timestamp.is_none() {
            return false;
        }

        if (chrono::Local::now() - variant.start_timestamp.unwrap())
            > chrono::Duration::new(test_settings.test_duration_minutes * 60, 0).unwrap()
        {
            return true;
        }

        false
    }

    /// Содержит ли вариант ещё неотвеченные вопросы.
    fn is_next_question(&self, username: &String, testname: &String) -> bool {
        if self.is_user_have_opened_variant(username, testname) {
            let current_question = self.variants[username].current_question;
            if current_question.is_none() {
                return true;
            }
            return current_question.unwrap() < self.variants[username].questions.len();
        }
        false
    }

    /// Сохранить ответ пользователя на последний неотвеченный вопрос.
    fn _put_answer(&mut self, username: &String, testname: &String, answer: &Answer) {
        if !self.is_user_have_opened_variant(username, testname) {
            eprintln!("ERROR: Test was done. Ignore put answer.");
            return;
        }

        let variant = self.variants.get_mut(username).unwrap();

        if variant.current_question.is_none() {
            eprintln!("ERROR: You answer on test that never started.");
            return;
        }

        variant.answers.push(answer.clone());
        variant.current_question = Some(variant.current_question.unwrap() + 1);
        if variant.answers.len() == variant.questions.len() {
            self.done_test(username, testname)
        }
    }

    /// Завершить тест
    fn done_test(&mut self, username: &String, testname: &String) {
        let mark = self.calculate_mark(username, testname);
        let start_time = self.variants[username].start_timestamp.unwrap().to_string();
        let end_time = chrono::Local::now().to_string();
        self.db
            .append_mark(username, testname, mark, &start_time, &end_time);
        self.variants.remove(username);
    }

    /// Посчитать оценку за тест.
    fn calculate_mark(&mut self, username: &String, testname: &String) -> f32 {
        if !self.is_user_have_opened_variant(username, testname) {
            eprintln!("ERROR: Test wan't run. Ignore calculate mark.");
            return -1.0;
        }

        let variant = self.variants.get_mut(username).unwrap();
        println!("{:?} {:?}", variant.questions, variant.answers);
        let mut result: f32 = 0.0;
        for i in 0..variant.answers.len() {
            if variant.questions[i].correct_answer == variant.answers[i] {
                result += 1.0;
            }
        }
        result
    }

    fn variant_collector(&mut self) {
        println!("Variant collector");
        let mut done_tests = vec![];

        for it in &self.variants {
            let variant = it.1;
            if self.is_test_time_is_over(&variant.username, &variant.testname) {
                done_tests.push((variant.username.clone(), variant.testname.clone()));
            }
        }

        for variant in done_tests {
            //self.done_test(&variant.0, &variant.1);
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
