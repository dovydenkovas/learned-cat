#![allow(unused)]

use diesel::prelude::*;
use dotenvy::dotenv;
use learned_cat_interfaces::schema::TestRecord;
use learned_cat_interfaces::Statistic;
use schema::{Test, User, Variant};
use std::env;
use std::process::exit;
use std::{collections::HashMap, path::PathBuf};

// use self::schema::Answer::dsl::*;
// use self::schema::Question::dsl::*;
// use self::schema::Test::dsl::*;
// use self::schema::Variant::dsl::*;

pub mod models;
pub mod schema;

use learned_cat_interfaces::{
    schema::{Answer, Question},
    settings::{Settings, TestSettings},
    Database,
};

pub struct TestDatabase {
    connection: SqliteConnection,
}

impl TestDatabase {
    pub fn new(database_url: String) -> TestDatabase {
        let connection = SqliteConnection::establish(&database_url).unwrap_or_else(|_| {
            eprintln!("Невозможно открыть Sqlite базу данных {}.", database_url);
            exit(1)
        });
        TestDatabase { connection }
    }
}

impl Statistic for TestDatabase {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String> {
        use self::schema::User::dsl::*;
        User.select(name).load(&mut self.connection).unwrap()
    }

    /// Список результатов конкретного пользователя.
    fn results(&mut self, username: &String) -> TestRecord {
        unimplemented!()
    }
}

impl Database for TestDatabase {
    /// Сколько попыток для прохождения теста testname потратил пользователь username.
    fn attempts_counter(&mut self, username: &String, testname: &String) -> u32 {
        Variant::table
            .left_join(User::table)
            .filter(User::dsl::name.eq(username))
            .left_join(Test::table)
            .filter(User::dsl::name.eq(testname))
            .select(Variant::dsl::begin_timestamp)
            .count()
            .get_result::<i64>(&mut self.connection)
            .unwrap() as u32
    }

    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32> {
        Variant::table
            .left_join(User::table)
            .filter(User::dsl::name.eq(username))
            .left_join(Test::table)
            .filter(User::dsl::name.eq(testname))
            .select(Variant::dsl::mark)
            .load::<f32>(&mut self.connection)
            .unwrap()
    }

    /// Сохранить баллы за тест testname для пользователя username.
    fn append_mark(&mut self, username: &String, testname: &String, mark: f32) {
        if User::table
            .filter(User::dsl::name.eq(username))
            .count()
            .get_result::<i64>(&mut self.connection)
            .unwrap()
            == 0
        {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_database() {
        let db_path = "/tmp/lc_init_database.db";
        let mut db = TestDatabase::new(db_path.to_string());
        std::fs::remove_file(db_path).unwrap();
    }

    fn fill_database(db: &mut TestDatabase) {
        db.append_mark(&"vlad".to_string(), &"math".to_string(), 5.0);
        db.append_mark(&"sveta".to_string(), &"math".to_string(), 8.8);
        db.append_mark(&"artem".to_string(), &"history".to_string(), 4.83);
        db.append_mark(&"vlad".to_string(), &"marh".to_string(), 3.2);
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

        assert_eq!(db.users(), vec!["vlad", "artem", "sveta"]);

        std::fs::remove_file(db_path).unwrap();
    }

    #[test]
    fn attempts_couter() {
        let db_path = "/tmp/lc_users.db";
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
}
