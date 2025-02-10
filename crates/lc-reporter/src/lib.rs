use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod csv_reporter;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TestRecord {
    pub username: String,
    pub testname: String,
    pub mark: f32,
    pub end_datetime: chrono::DateTime<chrono::FixedOffset>,
    pub start_datetime: chrono::DateTime<chrono::FixedOffset>,
}

pub trait Reporter {
    /// Сохранение результатов тестирования в файл.
    fn save_report(&mut self, filename: PathBuf);
}

pub trait Statistic {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String>;

    /// Список результатов конкретного пользователя.
    fn results(&mut self, username: &String) -> Vec<TestRecord>;
}
