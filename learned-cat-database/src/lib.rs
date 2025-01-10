#![allow(unused)]

use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::{collections::HashMap, path::PathBuf};

// use self::schema::Answer::dsl::*;
// use self::schema::Question::dsl::*;
// use self::schema::Test::dsl::*;
// use self::schema::Variant::dsl::*;
use self::schema::User::dsl::*;

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
    pub fn new(_tests_directory_path: PathBuf) -> TestDatabase {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        TestDatabase { connection }
    }
}

impl Database for TestDatabase {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String> {
        unimplemented!();
    }

    /// Сколько попыток для прохождения теста testname потратил пользователь username.
    fn attempts_counter(&mut self, username: &String, testname: &String) -> u32 {
        unimplemented!();
    }

    /// Получить баллы за тест testname для пользователя username.
    fn marks(&mut self, username: &String, testname: &String) -> Vec<f32> {
        unimplemented!();
    }

    /// Сохранить баллы за тест testname для пользователя username.
    fn append_mark(&mut self, username: &String, testname: &String, mark: f32) {
        unimplemented!();
    }
}
