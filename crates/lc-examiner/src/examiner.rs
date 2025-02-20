use std::collections::HashMap;

use log::{debug, error};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::network::Marks;
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
                marks: self.get_marks(username, testname),
            };
        }

        // Отправить описание теста.
        Response::TestStarted {
            banner: self.config.test_banner(testname).unwrap_or("".to_string()),
        }
    }

    /// Отправить результаты тестирования.
    fn get_marks(&mut self, username: &String, testname: &String) -> Marks {
        let marks = self.db.marks(username, testname);
        if marks.is_empty() {
            return Marks::Empty;
        }

        if self.config.test_settings(testname).unwrap().show_results {
            Marks::Marks { marks }
        } else {
            Marks::Done
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
                tests.push((test.clone(), self.get_marks(username, test)));
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
                marks: self.get_marks(username, testname),
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
                marks: self.get_marks(username, testname),
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
                marks: self.get_marks(username, testname),
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
        self.db.append_mark(
            username,
            testname,
            mark,
            &start_time,
            &end_time,
            &self.variants[username],
        );
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
            result += check_answer(&variant.answers[i], &variant.questions[i].correct_answer);
        }
        result
    }
}

/// Проверка корректности ответа
/// За правильный ответ начисляется `1 / n_true` баллов,
/// где `n_true` - количество правильных ответов.
/// Неправильный ответ нивелирует один правильный.
/// При этом общий балл за вопрос не может быть меньше нуля или больше единицы.
fn check_answer(answer: &Answer, correct_answer: &Answer) -> f32 {
    let correct_answer = correct_answer.as_array();
    let answer = answer.as_array();
    let d = 1.0 / correct_answer.len() as f32;

    let mut mark = 0.0;
    for a in &answer {
        if correct_answer.contains(a) {
            mark += d;
        } else {
            mark -= d;
        }
    }

    mark.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use crate::network::Marks;
    use crate::{
        network::Response,
        schema::{Answer, Question},
        settings::{self},
        Config, Database,
    };

    use super::{check_answer, Examiner};

    struct TDatabase {}

    impl Database for TDatabase {
        fn attempts_counter(&mut self, _username: &String, _testname: &String) -> u32 {
            1
        }

        fn marks(&mut self, _username: &String, _testname: &String) -> Vec<f32> {
            vec![3.0]
        }

        /// Сохранить баллы за тест testname для пользователя username.
        fn append_mark(
            &mut self,
            _username: &String,
            _testname: &String,
            _mark: f32,
            _start_timestamp: &String,
            _end_timestamp: &String,
        ) {
        }
    }

    struct TConfig {}
    impl Config for TConfig {
        fn has_user(&self, username: &String) -> bool {
            *username == "student".to_string()
        }

        fn has_test(&self, testname: &String) -> bool {
            *testname == "math".to_string()
        }

        fn test_settings(&self, testname: &String) -> Option<settings::TestSettings> {
            if *testname == "math".to_string() {
                Some(settings::TestSettings {
                    caption: "math".to_string(),
                    questions_number: 1,
                    test_duration_minutes: 1,
                    number_of_attempts: 3,
                    show_results: true,
                    allowed_users: Some(vec!["student".to_string()]),
                    allowed_users_path: None,
                })
            } else {
                None
            }
        }

        fn test_banner(&self, testname: &String) -> Option<String> {
            if *testname == "math".to_string() {
                Some("description".to_string())
            } else {
                None
            }
        }

        fn question(&self, testname: &String, question_id: usize) -> Option<Question> {
            if *testname == "math".to_string() && question_id == 0 {
                Some(Question {
                    question: "2+2".to_string(),
                    answers: vec!["4".to_string(), "5".to_string()],
                    correct_answer: Answer::new(vec![0]),
                })
            } else {
                None
            }
        }

        /// Получить количество вопросов в тесте.
        fn questions_count(&self, testname: &String) -> Option<usize> {
            if *testname == "math".to_string() {
                Some(1)
            } else {
                None
            }
        }

        fn answer(&self, testname: &String, question_id: usize) -> Option<Answer> {
            if *testname == "math".to_string() && question_id == 0 {
                Some(Answer::new(vec![0]))
            } else {
                None
            }
        }

        fn has_access(&self, username: &String, testname: &String) -> bool {
            *username == "student".to_string() && *testname == "math".to_string()
        }

        fn user_tests_list(&self, _username: &String) -> Vec<String> {
            vec!["math".to_string()]
        }

        fn settings(&self) -> settings::Settings {
            settings::Settings {
                tests_directory_path: "example-config".to_string(),
                result_path: "marks.db".to_string(),
                server_address: "127.0.0.1:8080".to_string(),
                tests: vec![self.test_settings(&"math".to_string()).unwrap()],
                log_level: "debug".to_string(),
            }
        }
    }

    fn get_examiner() -> Examiner {
        let config = TConfig {};
        let database = TDatabase {};
        Examiner::new(Box::new(config), Box::new(database))
    }

    #[test]
    fn examiner_description() {
        let mut examiner = get_examiner();
        let resp = examiner.banner_to_start_test(&"student".to_string(), &"math".to_string());
        assert_eq!(
            resp,
            Response::TestStarted {
                banner: "description".to_string()
            }
        );

        let resp = examiner.banner_to_start_test(&"student2".to_string(), &"math".to_string());
        assert_eq!(resp, Response::NotAllowedUser);

        let resp = examiner.banner_to_start_test(&"student".to_string(), &"math2".to_string());
        assert_eq!(resp, Response::NotAllowedUser);

        let resp = examiner.banner_to_start_test(&"student2".to_string(), &"math2".to_string());
        assert_eq!(resp, Response::NotAllowedUser);
    }

    #[test]
    fn examiner_avaliable_tests() {
        let mut examiner = get_examiner();
        let resp = examiner.avaliable_tests(&"username".to_string());
        assert_eq!(resp, Response::NotAllowedUser);

        let resp = examiner.avaliable_tests(&"student".to_string());
        assert_eq!(
            resp,
            Response::AvaliableTests {
                tests: vec![("math".to_string(), Marks::Marks { marks: vec![3.0] })]
            }
        );
    }

    #[test]
    fn examiner_put_answer() {
        let mut examiner = get_examiner();
        let resp = examiner.put_answer(
            &"student".to_string(),
            &"math".to_string(),
            &Answer::new(vec![1]),
        );

        assert_eq!(
            resp,
            Response::End {
                marks: Marks::Marks { marks: vec![3.0] }
            }
        );

        let resp = examiner.put_answer(
            &"student2".to_string(),
            &"math".to_string(),
            &Answer::new(vec![1]),
        );

        assert_eq!(resp, Response::NotAllowedUser);

        let resp = examiner.put_answer(
            &"student".to_string(),
            &"math2".to_string(),
            &Answer::new(vec![1]),
        );

        assert_eq!(resp, Response::NotAllowedUser);
    }

    #[test]
    fn examiner_next_question() {
        let mut examiner = get_examiner();
        let resp = examiner.next_question(&"username".to_string(), &"math".to_string());
        assert_eq!(resp, Response::NotAllowedUser);

        let resp = examiner.next_question(&"student".to_string(), &"testname".to_string());
        assert_eq!(resp, Response::NotAllowedUser);

        let resp = examiner.next_question(&"student".to_string(), &"math".to_string());
        let true_resp = Response::NextQuestion {
            question: "2+2".to_string(),
            answers: vec!["4".to_string(), "5".to_string()],
        };
        assert_eq!(resp, true_resp);
    }

    #[test]
    fn test_check_answer() {
        assert_eq!(
            check_answer(&Answer::new(vec![0]), &Answer::new(vec![0])),
            1.0
        );
        assert_eq!(
            check_answer(&Answer::new(vec![1]), &Answer::new(vec![0])),
            0.0
        );

        assert_eq!(
            check_answer(&Answer::new(vec![0, 1]), &Answer::new(vec![1])),
            0.0
        );
        assert_eq!(
            check_answer(&Answer::new(vec![0]), &Answer::new(vec![0, 1])),
            0.5
        );
        assert_eq!(
            check_answer(&Answer::new(vec![0, 1]), &Answer::new(vec![0, 1])),
            1.0
        );
        assert_eq!(
            check_answer(&Answer::new(vec![0, 1, 2]), &Answer::new(vec![0, 1])),
            0.5
        );
        assert_eq!(
            check_answer(&Answer::new(vec![0, 1, 2, 3]), &Answer::new(vec![0, 1])),
            0.0
        );
        assert_eq!(
            check_answer(&Answer::new(vec![0, 1, 2, 3, 4]), &Answer::new(vec![0, 1])),
            0.0
        );
    }
}
