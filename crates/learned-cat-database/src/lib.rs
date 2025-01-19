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
        let connection = SqliteConnection::establish(&database_url).unwrap_or_else(|_| {
            eprintln!("Невозможно открыть Sqlite базу данных {}.", database_url);
            exit(1)
        });
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
            .filter(User::dsl::name.eq(testname))
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
            .filter(User::dsl::name.eq(testname))
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

        let mut test_id = Variant
            .filter(start_timestamp.eq(&start_time))
            .select(id)
            .get_result::<i32>(&mut self.connection);
        if test_id.is_ok() {
            return;
        }

        let user_id = self.append_user(username.clone());
        let test_id = self.append_test(testname.clone());

        insert_into(Variant)
            .values((
                schema::Variant::user.eq(user_id),
                schema::Variant::test.eq(test_id),
                schema::Variant::mark.eq(mark_value),
                schema::Variant::start_timestamp.eq(start_time.clone()),
                schema::Variant::end_timestamp.eq(end_time.clone()),
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
        db.append_mark(&"vlad".to_string(), &"math".to_string(), 5.0, "0", "1");
        db.append_mark(&"sveta".to_string(), &"math".to_string(), 8.8, "0", "1");
        db.append_mark(&"artem".to_string(), &"history".to_string(), 4.83, "0", "1");
        db.append_mark(&"vlad".to_string(), &"marh".to_string(), 3.2, "0", "1");
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
