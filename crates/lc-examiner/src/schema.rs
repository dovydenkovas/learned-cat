use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Answer {
    answers: Vec<usize>,
}

impl Answer {
    pub fn new(answer: Vec<usize>) -> Answer {
        let mut answers = answer.clone();
        answers.sort();
        Answer { answers }
    }

    pub fn push(&mut self, answer: usize) {
        self.answers.push(answer);
        self.answers.sort();
    }

    pub fn as_array(&self) -> Vec<usize> {
        self.answers.clone()
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Question {
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answer: Answer,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Variant {
    pub username: String,
    pub testname: String,
    pub start_timestamp: chrono::DateTime<chrono::Local>,
    pub questions: Vec<Question>,
    pub answers: Vec<Answer>,
}
