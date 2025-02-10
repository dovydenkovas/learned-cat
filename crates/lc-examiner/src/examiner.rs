use std::collections::HashMap;

use log::{debug, error};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::schema::{Answer, Variant};
use crate::{network::Response, schema::Question};
use crate::{Config, Database};

pub struct Examiner {
    config: Box<dyn Config>,
    db: Box<dyn Database>,
    /// Хранилище вариантов - username - variants
    variants: HashMap<String, Variant>,
}

impl Examiner {
    pub fn new(config: Box<dyn Config>, db: Box<dyn Database>) -> Examiner {
        let examiner = Examiner {
            config,
            db,
            variants: HashMap::new(),
        };
        examiner
    }

    /// Показать описание теста перед запуском
    pub fn banner_to_start_test(&mut self, username: &String, testname: &String) -> Response {
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
    pub fn avaliable_tests(&mut self, username: &String) -> Response {
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
    pub fn put_answer(
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
        self.next_question(username, testname)
    }

    /// Запустить тест или отправить новый вопрос.
    pub fn next_question(&mut self, username: &String, testname: &String) -> Response {
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

    pub fn variant_collector(&mut self) {
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
}
