use diesel::{insert_into, prelude::*};
use lc_reporter::{AnswerRecord, QuestionRecord, VariantRecord};
use lc_reporter::{MarkRecord, Statistic};
use log::error;
use std::collections::HashMap;
use std::process::exit;

pub mod models;
pub mod schema;

use crate::models::*;
use crate::schema::*;

use lc_examiner::Database;

pub struct TestDatabase {
    connection: SqliteConnection,
}

impl TestDatabase {
    pub fn new(database_url: String) -> TestDatabase {
        let mut connection = SqliteConnection::establish(&database_url).unwrap_or_else(|_| {
            error!("Невозможно открыть Sqlite базу данных {}.", database_url);
            exit(1)
        });

        let _ = diesel::sql_query(
            r#"CREATE TABLE users (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            name VARCHAR NOT NULL
            );"#,
        )
        .execute(&mut connection);

        let _ = diesel::sql_query(
            r#"
        CREATE TABLE tests (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            caption VARCHAR NOT NULL
            );"#,
        )
        .execute(&mut connection);

        let _ = diesel::sql_query(
            r#"
        CREATE TABLE variants (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            test_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            mark FLOAT NOT NULL,
            start_timestamp VARCHAR NOT NULL,
            end_timestamp VARCHAR NOT NULL
        );"#,
        )
        .execute(&mut connection);

        let _ = diesel::sql_query(
            r#"
        CREATE TABLE questions (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            variant_id NOT NULL,
            text TEXT NOT NULL
        );"#,
        )
        .execute(&mut connection);

        let _ = diesel::sql_query(
            r#"
        CREATE TABLE answers (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            question_id INTEGER NOT NULL,
            text TEXT NOT NULL,
            is_correct BOOLEAN,
            is_selected BOOLEAN
        );"#,
        )
        .execute(&mut connection);
        TestDatabase { connection }
    }

    fn append_user(&mut self, username: String) -> i32 {
        let mut user_id = users::dsl::users
            .filter(users::name.eq(&username))
            .select(users::id)
            .get_result::<i32>(&mut self.connection);

        if user_id.is_err() {
            let user: User = insert_into(users::table)
                .values((users::name.eq(username.clone()),))
                .get_result(&mut self.connection)
                .unwrap();
            user_id = Ok(user.id);
        }

        user_id.unwrap()
    }

    fn append_test(&mut self, testname: String) -> i32 {
        let mut test_id = tests::dsl::tests
            .filter(tests::caption.eq(&testname))
            .select(tests::id)
            .get_result::<i32>(&mut self.connection);

        if test_id.is_err() {
            let test: Test = insert_into(tests::table)
                .values((tests::caption.eq(testname.clone()),))
                .get_result(&mut self.connection)
                .unwrap();
            test_id = Ok(test.id);
        }

        test_id.unwrap()
    }

    fn append_questions(&mut self, variant_id: i32, variant: &lc_examiner::schema::Variant) {
        for i in 0..variant.answers.len() {
            let question = variant.questions[i].clone();

            let added_question: Question = insert_into(questions::table)
                .values((
                    questions::text.eq(question.question.clone()),
                    questions::variant_id.eq(variant_id),
                ))
                .get_result(&mut self.connection)
                .unwrap();

            let question_id = added_question.id; 
            let answers_arr = variant.answers[i].as_array();
            let mut insertable = vec![];

            for j in 0..question.answers.len() {
                    insertable.push((
                        answers::text.eq(question.answers[j].clone()),
                        answers::question_id.eq(question_id),
                        answers::is_selected.eq(answers_arr.contains(&j)),
                        answers::is_correct.eq(question.correct_answer.as_array().contains(&j))));
            }
                insert_into(answers::table)
                    .values(&insertable)
                    .execute(&mut self.connection)
                    .unwrap();

        }
    }

    fn get_questions_records(&mut self, variant_id: i32) -> Vec<QuestionRecord> {
        let answers = answers::table
            .inner_join(questions::table)
            .filter(questions::variant_id.eq(variant_id))
            .select((Question::as_select(), Answer::as_select())) //, Test::as_select()))
            .load::<(Question, Answer)>(&mut self.connection)
            .unwrap();

        let mut questions: HashMap<i32, QuestionRecord> = HashMap::new();
        for p in answers {
            let answer: Answer = p.1;
            let question: Question = p.0;

            if !questions.contains_key(&question.id) {
                questions.insert(
                    question.id,
                    QuestionRecord {
                        question: question.text,
                        answers: vec![],
                    },
                );
            }

            questions
                .get_mut(&question.id)
                .unwrap()
                .answers
                .push(AnswerRecord {
                    answer: answer.text,
                    is_correct: answer.is_correct,
                    is_selected: answer.is_selected,
                })
        }
        let mut result = vec![];
        for question in questions {
            result.push(question.1);
        }
        result
    }
}

impl Statistic for TestDatabase {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String> {
        users::table
            .select(users::name)
            .load(&mut self.connection)
            .unwrap()
    }

    /// Список результатов конкретного пользователя.
    fn results(&mut self, username: &String) -> Vec<MarkRecord> {
        let variants_req = variants::table
            .inner_join(users::table)
            .filter(users::name.eq(username))
            .inner_join(tests::table)
            .select((Variant::as_select(), User::as_select(), Test::as_select())) //, Test::as_select()))
            .load::<(Variant, User, Test)>(&mut self.connection)
            .unwrap();

        let mut results = Vec::<MarkRecord>::new();
        for variant in variants_req {
            let start_datetime = chrono::DateTime::parse_from_str(
                variant.0.start_timestamp.as_str(),
                "%Y-%m-%d %H:%M:%S.%f %z",
            )
            .unwrap();
            let end_datetime = chrono::DateTime::parse_from_str(
                variant.0.end_timestamp.as_str(),
                "%Y-%m-%d %H:%M:%S.%f %z",
            )
            .unwrap();

            results.push(MarkRecord {
                username: variant.1.name,
                testname: variant.2.caption,
                mark: variant.0.mark,
                end_datetime,
                start_datetime,
            });
        }

        results
    }

    /// Ответы пользователя на вопросы одного теста
    fn variants(&mut self, username: &String, testname: &String) -> Vec<VariantRecord> {
        let variants_req = variants::table
            .inner_join(users::table)
            .filter(users::name.eq(username))
            .inner_join(tests::table)
            .filter(tests::caption.eq(testname))
            .select((Variant::as_select(), User::as_select(), Test::as_select())) //, Test::as_select()))
            .load::<(Variant, User, Test)>(&mut self.connection)
            .unwrap();

        let mut results = Vec::<VariantRecord>::new();
        for variant in variants_req {
            let start_datetime = chrono::DateTime::parse_from_str(
                variant.0.start_timestamp.as_str(),
                "%Y-%m-%d %H:%M:%S.%f %z",
            )
            .unwrap();
            let end_datetime = chrono::DateTime::parse_from_str(
                variant.0.end_timestamp.as_str(),
                "%Y-%m-%d %H:%M:%S.%f %z",
            )
            .unwrap();

            let mark = variant.0.mark;
            let questions = self.get_questions_records(variant.0.id);

            results.push(VariantRecord {
                mark,
                end_datetime,
                start_datetime,
                questions,
            });
        }
        results
    }
}

impl Database for TestDatabase {
    /// Сколько попыток для прохождения теста testname потратил пользователь username.
    fn attempts_counter(&mut self, username: &String, testname: &String) -> u32 {
        variants::table
            .left_join(users::table)
            .filter(users::name.eq(username))
            .left_join(tests::table)
            .filter(tests::caption.eq(testname))
            .select(variants::start_timestamp)
            .count()
            .get_result::<i64>(&mut self.connection)
            .unwrap() as u32
    }

    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32> {
        variants::table
            .left_join(users::table)
            .filter(users::name.eq(username))
            .left_join(tests::table)
            .filter(tests::caption.eq(testname))
            .select(variants::mark)
            .load::<f32>(&mut self.connection)
            .unwrap()
    }

    /// Сохранить баллы за тест testname для пользователя username.
    fn append_mark(
        &mut self,
        username: &String,
        testname: &String,
        mark_value: f32,
        start_time: &String,
        end_time: &String,
        variant: &lc_examiner::schema::Variant,
    ) {
        let user_id_f = self.append_user(username.clone());
        let test_id_f = self.append_test(testname.clone());
        let mark_id = variants::dsl::variants
            .filter(variants::start_timestamp.eq(&start_time))
            .filter(variants::end_timestamp.eq(&end_time))
            .filter(variants::user_id.eq(user_id_f))
            .filter(variants::test_id.eq(test_id_f))
            .select(variants::id)
            .get_result::<i32>(&mut self.connection);
        if mark_id.is_err() {
            insert_into(variants::table)
                .values((
                    variants::user_id.eq(user_id_f),
                    variants::test_id.eq(test_id_f),
                    variants::mark.eq(mark_value),
                    variants::start_timestamp.eq(start_time.clone()),
                    variants::end_timestamp.eq(end_time.clone()),
                ))
                .execute(&mut self.connection)
                .unwrap();
        }
        let mark_id = variants::dsl::variants
            .filter(variants::start_timestamp.eq(&start_time))
            .filter(variants::end_timestamp.eq(&end_time))
            .filter(variants::user_id.eq(user_id_f))
            .filter(variants::test_id.eq(test_id_f))
            .select(variants::id)
            .get_result::<i32>(&mut self.connection)
            .unwrap();

        self.append_questions(mark_id, variant);
    }
}

#[cfg(test)]
mod db_tests {
    use super::*;

    #[test]
    fn init_database() {
        let db_path = "/tmp/lc_init_database.db";
        let _db = TestDatabase::new(db_path.to_string());
        std::fs::remove_file(db_path).unwrap();
    }

    fn fill_database(db: &mut TestDatabase) {
        let start_time = "2025-01-26 13:33:41.789001340 +03:00".to_string();
        let end_time = "2025-01-26 13:33:41.789001340 +03:00".to_string();

        db.append_mark(
            &"vlad".to_string(),
            &"math".to_string(),
            5.0,
            &start_time,
            &end_time,
            &lc_examiner::schema::Variant {
                username: "vlad".to_string(),
                testname: "math".to_string(),
                start_timestamp: chrono::offset::Local::now(),
                questions: vec![],
                answers: vec![],
            },
        );

        db.append_mark(
            &"sveta".to_string(),
            &"math".to_string(),
            8.8,
            &start_time,
            &end_time,
            &lc_examiner::schema::Variant {
                username: "sveta".to_string(),
                testname: "math".to_string(),
                start_timestamp: chrono::offset::Local::now(),
                questions: vec![],
                answers: vec![],
            },
        );
        let start_datetime = "2025-01-26 13:33:41.789001340 +03:00".to_string();

        let end_datetime = "2025-01-26 13:33:44.698762199 +03:00".to_string();
        db.append_mark(
            &"artem".to_string(),
            &"history".to_string(),
            4.83,
            &start_datetime,
            &end_datetime,
            &lc_examiner::schema::Variant {
                username: "artem".to_string(),
                testname: "history".to_string(),
                start_timestamp: chrono::offset::Local::now(),
                questions: vec![],
                answers: vec![],
            },
        );
        let start_time = "5".to_string();
        let end_time = "6".to_string();
        db.append_mark(
            &"vlad".to_string(),
            &"math".to_string(),
            3.2,
            &start_time,
            &end_time,
            &lc_examiner::schema::Variant {
                username: "vlad".to_string(),
                testname: "math".to_string(),
                start_timestamp: chrono::offset::Local::now(),
                questions: vec![],
                answers: vec![],
            },
        );
    }

    #[test]
    fn marks() {
        let db_path = "/tmp/lc_marks.db";
        let mut db = TestDatabase::new(db_path.to_string());

        fill_database(&mut db);

        assert_eq!(db.marks(&"artem".to_string(), &"math".to_string()), vec![]);
        assert_eq!(
            db.marks(&"vlad".to_string(), &"math".to_string()),
            vec![5.0, 3.2]
        );
        assert_eq!(
            db.marks(&"sveta".to_string(), &"math".to_string()),
            vec![8.8]
        );

        std::fs::remove_file(db_path).unwrap();
    }

    #[test]
    fn users() {
        let db_path = "/tmp/lc_users.db";
        let mut db = TestDatabase::new(db_path.to_string());

        assert_eq!(db.users(), vec![] as Vec<String>);
        fill_database(&mut db);

        let mut users = db.users();
        users.sort();

        assert_eq!(users, vec!["artem", "sveta", "vlad"]);

        std::fs::remove_file(db_path).unwrap();
    }

    #[test]
    fn attempts_couter() {
        let db_path = "/tmp/lc_attempts_counter.db";
        let mut db = TestDatabase::new(db_path.to_string());

        assert_eq!(
            db.attempts_counter(&"vlad".to_string(), &"math".to_string()),
            0
        );
        fill_database(&mut db);

        assert_eq!(
            db.attempts_counter(&"vlad".to_string(), &"math".to_string()),
            2
        );
        assert_eq!(
            db.attempts_counter(&"vlad".to_string(), &"history".to_string()),
            0
        );
        assert_eq!(
            db.attempts_counter(&"sveta".to_string(), &"math".to_string()),
            1
        );
        assert_eq!(
            db.attempts_counter(&"artem".to_string(), &"history".to_string()),
            1
        );

        std::fs::remove_file(db_path).unwrap();
    }

    #[test]
    fn statistic() {
        let db_path = "/tmp/lc_statistic.db";
        let mut db = TestDatabase::new(db_path.to_string());

        assert_eq!(db.results(&"artem".to_string()), vec![] as Vec<MarkRecord>);
        fill_database(&mut db);

        let res = db.results(&"artem".to_string());
        let start_datetime = chrono::DateTime::parse_from_str(
            "2025-01-26 13:33:41.789001340 +03:00",
            "%Y-%m-%d %H:%M:%S.%f %z",
        )
        .unwrap();
        let end_datetime = chrono::DateTime::parse_from_str(
            "2025-01-26 13:33:44.698762199 +03:00",
            "%Y-%m-%d %H:%M:%S.%f %z",
        )
        .unwrap();

        let expected = vec![MarkRecord {
            username: "artem".to_string(),
            testname: "history".to_string(),
            mark: 4.83,
            start_datetime,
            end_datetime,
        }];
        assert_eq!(res, expected);

        std::fs::remove_file(db_path).unwrap();
    }
}
