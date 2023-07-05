use std::io::{self, Read};
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use serde_json::from_str as from_json;
use serde_derive::Deserialize;
use markdown_parser::read_file;


#[derive(Debug)]
struct Quest {
    name: String,
    banner: String,
    questions: Vec<Question>,
}


#[derive(Debug)]
struct Question {
    question: String, 
    answers: Vec<String>,
    correct_answers: Vec<usize>
}


#[derive(Deserialize)]
struct Settings {
    quests_paths: Vec<String>
}


fn read_settings() -> Result<Settings, Box<dyn Error>> {
    let mut file = File::open("settings.json")?;
    let mut settings = String::new();
    file.read_to_string(&mut settings)?;
    Ok(from_json(&settings)?)
}


fn main() {
    println!("Сервер"); 
    
     let settings = match read_settings() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Не могу прочитать файл конфигурации settings.json: {:?}", err.to_string());
            return;
        }
    };

    let mut quests: Vec<Quest> = vec![];
    

    for test in settings.quests_paths {
        println!("{}", test);
        
        let file = File::open(test).unwrap();
        let file = BufReader::new(file);
        
        let mut name = String::new();
        let mut banner = String::new();
        let mut questions: Vec<Question> = vec![];
        let mut answer_number: usize = 1;
        
        let mut question = Question { 
            question: "".to_string(), 
            answers: vec![], 
            correct_answers: vec![] 
        };

        let mut is_name = true;
        let mut is_banner = true;
        let mut is_question = true;

        for line in file.lines() {
            let line = line.unwrap();
            if is_name || is_banner {
                if line.starts_with("#") {
                    if is_name {
                        name = line
                            .to_string()
                            .split("#")
                            .last()
                            .unwrap()
                            .trim()
                            .to_string();
                        is_name = false;
                    } else {
                        is_banner = false;
                    }
                } else {
                    banner += &line;
                }
            }


            if !is_banner && ! is_name {
                if line.starts_with("#") || is_question {
                    if line.starts_with("*") || line.starts_with("+") {
                        is_question = false;
                    } else {
                        if line.starts_with("#") {
                            is_question = true;
                            
                            questions.push(question);

                            question = Question { 
                                question: "".to_string(), 
                                answers: vec![], 
                                correct_answers: vec![] 
                            };
                            
                            name = String::new();
                            banner = String::new();
                            //questions = vec![];
                            answer_number = 1;

                        }
                        question.question += line
                            .to_string()
                            .split("#")
                            .last()
                            .unwrap()
                            .trim();
                    }
                } else {
                    if line.starts_with("*") {
                        question.answers.push(line);
                        answer_number += 1;
                    } else if line.starts_with("+") {
                        question.answers.push(line);
                        question.correct_answers.push(answer_number);
                        answer_number += 1;
                    } else {
                        // TODO
                        let mut a = question
                            .answers
                            .last()
                            .unwrap();
                    }
                }
            }
        }
        quests.push(Quest {
            name: name,
            banner: banner,
            questions: questions
        });
    }

    println!("{:?}", quests);
}
