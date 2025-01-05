use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::thread_rng;

use learned_cat_interfaces::schema::Variant;
use learned_cat_interfaces::{
    network::{Command, Response},
    schema::Question,
};
use learned_cat_interfaces::{Database, Server};

pub struct Examiner {
    db: Box<dyn Database>,
    srv: Box<dyn Server>,
    /// Хранилище вариантов - username - variants
    variants: HashMap<String, Variant>,
}

impl Examiner {
    pub fn new(db: Box<dyn Database>, srv: Box<dyn Server>) -> Examiner {
        let examiner = Examiner {
            db,
            srv,
            variants: HashMap::new(),
        };
        examiner
    }

    /// Главный цикл обработки запросов.
    pub fn mainloop(&mut self) {
        loop {
            match self.srv.pop_request() {
                Some(request) => {
                    let response = match request.command {
                        Command::StartTest | Command::GetNextQuestion => {
                            self.next_question(&request.user, &request.test)
                        }

                        Command::GetAvaliableTests => self.avaliable_tests(&request.user),

                        Command::PutAnswer { answer } => {
                            self.put_answer(&request.user, &request.test, &answer)
                        }
                    };
                    self.srv.push_response(response);
                }
                None => {
                    self.variant_collector();
                }
            }
        }
    }

    /// Предоставить список доступных тестов.
    fn avaliable_tests(&self, username: &String) -> Response {
        if !self.db.has_user(username) {
            Response::NotAllowedUser
        } else {
            Response::AvaliableTests {
                tests: self.db.user_tests_list(username),
            }
        }
    }

    /// Сохранить ответ на вопрос.
    pub fn put_answer(
        &mut self,
        username: &String,
        testname: &String,
        answer: &Vec<usize>,
    ) -> Response {
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
        if !self.db.has_access(username, testname) {
            return Response::NotAllowedUser;
        }

        if self.db.remaining_attempts_number(username, testname) <= 0 {
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
            banner: self.db.test_banner(testname),
        }
    }

    /// Возвращает первый неотвеченный вопрос.
    fn get_next_question(&mut self, username: &String) -> Response {
        let variant = self.variants.get_mut(username).unwrap();
        if variant.timestamp.is_none() {
            variant.timestamp = Some(chrono::offset::Local::now());
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
        let test_settings = self.db.test_settings(testname);

        let mut vec: Vec<usize> = (0..test_settings.questions.len()).collect();
        vec.shuffle(&mut thread_rng());

        let mut questions: Vec<Question> = vec![];
        for i in 0..test_settings.questions_number {
            questions.push(test_settings.questions[vec[i]].clone());
        }

        Variant {
            username: username.clone(),
            testname: testname.clone(),
            timestamp: None,
            questions,
            answers: vec![],
            current_question: None,
            result: None,
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
        let test_settings = self.db.test_settings(testname);
        let variant = &self.variants[username];
        if variant.timestamp.is_none() {
            return false;
        }

        if (chrono::Local::now() - variant.timestamp.unwrap())
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
    fn _put_answer(&mut self, username: &String, testname: &String, answer: &Vec<usize>) {
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
        if variant.current_question.unwrap() == variant.questions.len() {
            self.done_test(username, testname)
        }
    }

    /// Завершить тест
    fn done_test(&mut self, username: &String, testname: &String) {
        let mark = self.calculate_mark(username, testname);
        self.db.add_mark(username, testname, mark);
        self.variants.remove(username);
    }

    /// Посчитать оценку за тест.
    fn calculate_mark(&mut self, username: &String, testname: &String) -> f32 {
        if !self.is_user_have_opened_variant(username, testname) {
            eprintln!("ERROR: Test wan't run. Ignore calculate mark.");
            return -1.0;
        }

        let variant = self.variants.get_mut(username).unwrap();
        let mut result: f32 = 0.0;
        for i in 0..variant.questions.len() {
            variant.answers[i].sort();
            if variant.questions[i].correct_answers == variant.answers[i] {
                result += 1.0;
            }
        }
        variant.result = Some(result);
        result
    }

    fn variant_collector(&mut self) {
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
