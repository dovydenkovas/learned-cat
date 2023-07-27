use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::model::{Test, Question};


enum ParseState {
    QuestCaption,
    QuestBanner,
    ReadQuestion,
    ReadAnswer,
}


/// Парсит Markdown файл тестирования
pub fn read_test(path: &Path, test: &mut Test) {
    let file = File::open(path).expect(format!("Не могу открыть файл теста: {:?}", path).as_str());
    let file = BufReader::new(file);
      
    let mut name = String::new();
    let mut banner = String::new();
    let mut questions: Vec<Question> = vec![];
    let mut answer_number: usize = 0;
    
    let mut question = Question { 
        question: "".to_string(), 
        answers: vec![], 
        correct_answers: vec![] 
    };

    let mut state = ParseState::QuestCaption;
    
    for line in file.lines() {
        let line = line.unwrap().trim().to_string();
        
        match state {
            ParseState::QuestCaption => {
                if line.starts_with("#") {
                    name = line
                    .to_string()
                    .split("#")
                    .last()
                    .unwrap()
                    .trim()
                    .to_string();
                    state = ParseState::QuestBanner;   
                }
            }
            
            ParseState::QuestBanner => {
                if line.starts_with("#") {
                    state = ParseState::ReadQuestion;
                } else {
                    banner += &line;
                }
            }

            ParseState::ReadQuestion => {
                if line.starts_with("*") || line.starts_with("+") {
                    state = ParseState::ReadAnswer; 
                }
            }

            ParseState::ReadAnswer => {
                if line.starts_with("#") {
                    state = ParseState::ReadQuestion;
                    if question.question.len() > 0 { 
                        questions.push(question);
                    }

                    question = Question { 
                        question: "".to_string(), 
                        answers: vec![], 
                        correct_answers: vec![] 
                    };
                    
                    answer_number = 0;
                }
            }
        }

        match state {
            ParseState::ReadQuestion => {
                    question.question += line
                        .to_string()
                        .split("#")
                        .last()
                        .unwrap()
                        .trim();
            }

            ParseState::ReadAnswer => {
                if line.starts_with("*") { // answer
                    question.answers.push(line[1..].trim().to_string());
                    answer_number += 1;
                } else if line.starts_with("+") { // true answer
                    question.answers.push(line[1..].trim().to_string());
                    question.correct_answers.push(answer_number);
                    answer_number += 1;
                } else { // multiline answer
                    let i: usize = question.answers.len();
                    if line.len() > 0 {
                        let answer = format!("{}\n  {}", question.answers[i-1], line);
                        question.answers[i-1] = answer;
                    }
                }
            }

            _ => ()
        }

    }

    if question.question.len() > 0 {
        questions.push(question);
    }

    if name.len() > 0 {
        test.caption = name;
    }
    test.banner = Some(banner);
    test.questions = Some(questions);
}


