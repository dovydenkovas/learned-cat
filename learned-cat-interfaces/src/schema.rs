use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Question {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Answer {
    pub answers: Vec<usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Variant {
    pub username: String,
    pub testname: String,
    pub timestamp: Option<chrono::DateTime<chrono::Local>>,
    pub questions: Vec<Question>,
    pub answers: Vec<Vec<usize>>,
    pub current_question: Option<usize>,
    pub result: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Variants {
    pub variants: Vec<Variant>,
}
