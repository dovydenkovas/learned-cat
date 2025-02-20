use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod csv_reporter;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct MarkRecord {
    pub username: String,
    pub testname: String,
    pub mark: f32,
    pub end_datetime: chrono::DateTime<chrono::FixedOffset>,
    pub start_datetime: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AnswerRecord {
    pub answer: String,
    pub is_correct: bool,
    pub is_selected: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct QuestionRecord {
    pub question: String,
    pub answers: Vec<AnswerRecord>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct VariantRecord {
    pub mark: f32,
    pub end_datetime: chrono::DateTime<chrono::FixedOffset>,
    pub start_datetime: chrono::DateTime<chrono::FixedOffset>,
    pub questions: Vec<QuestionRecord>,
}

pub trait Reporter {
    /// Сохранение результатов тестирования в файл.
    fn marks_report(&mut self, filename: PathBuf);

    /// Созранение вариантов пользователя в файл.
    fn variants_report(&mut self, username: &String, testname: &String);
}

pub trait Statistic {
    /// Список пользователей, закончивших хотя бы одну попытку.
    fn users(&mut self) -> Vec<String>;

    /// Список результатов конкретного пользователя.
    fn results(&mut self, username: &String) -> Vec<MarkRecord>;

    /// Ответы пользователя на вопросы одного теста
    fn variants(&mut self, username: &String, testname: &String) -> Vec<VariantRecord>;
}
