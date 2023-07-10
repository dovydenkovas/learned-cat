/// Клиентское приложение программы тестирования. 
/// Отправляет запросы на сервер и предоставляет пользовательский интерфейс. 

use std::error::Error;
use std::io::stdin;
use std::net::TcpStream;
use std::io::prelude::*;

use clap::Parser;
use whoami;

mod network_structs;
use network_structs::{Response, Request, Command, NextQuestion};



/// Структура аргументов командной строки.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Название теста.
    name: Option<String>,
    
    /// Отобразить доступные тесты.
    #[arg(short, long)]
    list: bool,
}


/// Парсит аргументы и запускает соответствующее действие.
fn main() {
    let cli = Cli::parse();
    if let Some(name) = cli.name {
        start_test(name);
    } else if cli.list {
        print_avaliable_tests();
    } else {
        println!("Для запуска действия укажите имя теста");
        print_avaliable_tests();
    }
}


/// Обслуживает процесс тестирования.
fn start_test(test_name: String) {
    println!("Ищу тест: {test_name}");
    
    let request = Request {
        user: whoami::username(),
        command: Command::StartTest { test: test_name.clone() } 
    };


    match send_request(&request) {
        Ok(response) => {
            match response {
                Response::StartTest { banner } => {
                    println!("{banner}");
                    let mut s = String::new();
                    let _ = stdin().read_line(&mut s);
                    println!("Вы готовы начать тестирование? (Введите <да> или <нет>)");
                    match s.to_lowercase().as_str() {
                        "yes" | "y" | "да" | "д" => run_test(test_name),
                        _ => (),
                    }
                },
                _ => eprintln!("Ошибка запуска теста."),
            }
        },
        Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string())
    }
}


fn run_test(test_name: String) {
    let next_question_request = Request {
        user: whoami::username(),
        command: Command::GetNextQuestion {
            test: test_name,
            previos_answer: vec![]
        }
    };

    loop {
        let response = send_request(&next_question_request);

        match response {
            Ok(Response::GetNextQuestion { question }) => {
                match question {
                    NextQuestion::Question { question, answers } => {
                        ask_question(question, answers);
                    },

                    NextQuestion::TheEnd { result } => {
                        println!("Тест завершен. Ваш результат: {}", result);
                        break;
                    }
                }
            },
            _ => ()
        }
    }
}

/// Задает вопрос 
fn ask_question(question: String, answers: Vec<String>) -> Vec<u8> {
    println!("{question}");

    for i in 0..answers.len() {
        println!("{}) {}", i+1, answers[i]);
    }

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer).expect("Не понимаю ответ");
    
    let mut answer: Vec<u8> = answer
        .replace(",", " ")
        .replace("  ", " ")
        .trim()
        .split(" ")
        .map(|x| x.parse::<u8>().unwrap())
        .collect();

    for i in 0..answer.len() {
        if answer[i] as usize <= answers.len() && answer[i] > 0 {
            answer[i] -= 1;
        }
    }

    answer
}



/// Выводит перечень тестов.
fn print_avaliable_tests() { 
    let request = network_structs::Request {
        user: whoami::username(),
        command: network_structs::Command::GetAvaliableTests
    };

    match send_request(&request) {
        Ok(response) => {
            match response {
                network_structs::Response::AvaliableTests { tests } => {
                    println!("Перечень доступных тестов:");
                    for test in tests {
                        println!("{test}");
                    }
                },
                _ => eprintln!("Ошибка чтения списка тестов."),
            }
        },
        Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string())
    }
}


/// Осуществляет связь с сервером.
fn send_request(request: &network_structs::Request)
    -> Result<network_structs::Response, Box<dyn Error>> {

    let request = bincode::serialize(&request)?;
    let mut response = [0 as u8; 5000];

    let mut stream = TcpStream::connect("127.0.0.1:65001")?;
    stream.write(&request)?;
    let n_bytes = stream.read(&mut response)?;
    
    Ok(bincode::deserialize::<network_structs::Response>(&response[..n_bytes])?)
}




