#![allow(unused)]

use diesel::{insert_into, prelude::*};
use dotenvy::dotenv;
use learned_cat_interfaces::schema::TestRecord;
use learned_cat_interfaces::Statistic;
use schema::{Test, User, Variant};
use std::env;
use std::process::exit;
use std::{collections::HashMap, path::PathBuf};

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
        let mut connection = SqliteConnection::establish(&database_url).unwrap_or_else(|_| {
            eprintln!("Невозможно открыть Sqlite базу данных {}.", database_url);
            exit(1)
        });

        diesel::sql_query(
            r#"CREATE TABLE User (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            name VARCHAR NOT NULL
            );"#,
        )
        .execute(&mut connection);
        diesel::sql_query(
            r#"
        CREATE TABLE Test (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            caption VARCHAR NOT NULL
            );"#,
        )
        .execute(&mut connection);
        diesel::sql_query(
            r#"
        CREATE TABLE Variant (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            test INTEGER NOT NULL,
            user INTEGER NOT NULL,
            mark FLOAT NOT NULL,
            start_timestamp VARCHAR NOT NULL,
            end_timestamp VARCHAR NOT NULL
        );"#,
        )
        .execute(&mut connection);

        TestDatabase { connection }
    }

    fn append_user(&mut self, username: String) -> i32 {
        use schema::User::dsl::*;

        let mut user_id = User
            .filter(name.eq(&username))
            .select(id)
            .get_result::<i32>(&mut self.connection);

        if user_id.is_err() {
            insert_into(User)
                .values((name.eq(username.clone()),))
                .execute(&mut self.connection)
                .unwrap();

            user_id = User
                .filter(name.eq(&username))
                .select(id)
                .get_result::<i32>(&mut self.connection);
        }

        user_id.unwrap()
    }

    fn append_test(&mut self, testname: String) -> i32 {
        use schema::Test::dsl::*;

        let mut test_id = Test
            .filter(caption.eq(&testname))
            .select(id)
            .get_result::<i32>(&mut self.connection);

        if test_id.is_err() {
            insert_into(Test)
                .values((caption.eq(testname.clone()),))
                .execute(&mut self.connection)
                .unwrap();

            test_id = Test
                .filter(caption.eq(&testname))
                .select(id)
                .get_result::<i32>(&mut self.connection);
        }

        test_id.unwrap()
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
            .filter(Test::dsl::caption.eq(testname))
            .select(Variant::dsl::start_timestamp)
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
            .filter(Test::dsl::caption.eq(testname))
            .select(Variant::dsl::mark)
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
    ) {
        use schema::Variant::dsl::*;
        let user_id = self.append_user(username.clone());
        let test_id = self.append_test(testname.clone());

        let mut mark_id = Variant
            .filter(start_timestamp.eq(&start_time))
            .filter(end_timestamp.eq(&end_time))
            .filter(user.eq(user_id))
            .filter(test.eq(test_id))
            .select(id)
            .get_result::<i32>(&mut self.connection);
        if mark_id.is_ok() {
            return;
        }

        insert_into(Variant)
            .values((
                user.eq(user_id),
                test.eq(test_id),
                mark.eq(mark_value),
                start_timestamp.eq(start_time.clone()),
                end_timestamp.eq(end_time.clone()),
            ))
            .execute(&mut self.connection)
            .unwrap();
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
        let start_time = "0".to_string();
        let end_time = "1".to_string();
        db.append_mark(
            &"vlad".to_string(),
            &"math".to_string(),
            5.0,
            &start_time,
            &end_time,
        );
        let start_time = "2".to_string();
        let end_time = "3".to_string();
        db.append_mark(
            &"sveta".to_string(),
            &"math".to_string(),
            8.8,
            &start_time,
            &end_time,
        );
        let start_time = "3".to_string();
        let end_time = "4".to_string();
        db.append_mark(
            &"artem".to_string(),
            &"history".to_string(),
            4.83,
            &start_time,
            &end_time,
        );
        let start_time = "5".to_string();
        let end_time = "6".to_string();
        db.append_mark(
            &"vlad".to_string(),
            &"math".to_string(),
            3.2,
            &start_time,
            &end_time,
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
}
