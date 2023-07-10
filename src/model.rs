/// Содержит структуры тестов
use serde::Deserialize;


#[derive(Debug)]
pub struct Quest {
    pub name: String,
    pub banner: String,
    pub questions: Vec<Question>,
}


#[derive(Debug)]
pub struct Question {
    pub question: String, 
    pub answers: Vec<String>,
    pub correct_answers: Vec<usize>
}


#[derive(Deserialize, Debug)]
pub struct Settings {
    pub quests_directory_path: String,
    pub quests_file_names: Vec<String>
}

