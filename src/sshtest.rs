/// Клиентское приложение программы тестирования. 
/// Отправляет запросы на сервер и предоставляет пользовательский интерфейс. 

use std::error::Error;
use std::io::stdin;
use std::net::TcpStream;
use std::io::prelude::*;

use clap::Parser;
use whoami;

mod network;
use network::{Response, Request, Command};



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
    let request = Request {
        user: whoami::username(),
        test: test_name.clone(),
        command: Command::StartTest 
    };


    match send_request(&request) {
        Ok(response) => {
            match response {
                Response::TestStarted { banner } => {
                    println!("{banner}");
                    println!("Вы готовы начать тестирование? (Введите <да> или <нет>)");

                    let mut s = String::new();
                    let _ = stdin().read_line(&mut s);
                    match s.trim().to_lowercase().as_str() {
                        "yes" | "y" | "да" | "д" => run_test(test_name),
                        _ => (),
                    }
                },
                
                Response::End { result } => {
                    println!("Тест завершен. Ваш результат: {}", result);
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
        test: test_name.clone(),
        command: Command::GetNextQuestion
    };

    loop {
        let response = send_request(&next_question_request);

        match response {
            Ok(Response::NextQuestion { question, answers }) => {
                let answers = ask_question(question, answers);
                let put_answer_request = Request {
                    user: whoami::username(),
                    test: test_name.clone(),
                    command: Command::PutAnswer { answer: answers } 
                };

                match send_request(&put_answer_request) {
                    Ok(Response::End { result }) => {
                        println!("Тест завершен. Ваш результат: {}", result);
                        break;
                    }, 

                    _ => (),
                }
                
            },
            
            Ok(Response::End { result }) => {
                println!("Тест завершен. Ваш результат: {}", result);
                break;
            },
            
            _ => ()
        }
    }
}

/// Задает вопрос 
fn ask_question(question: String, answers: Vec<String>) -> Vec<usize> { 
    'ask: loop {
        println!("");
        println!(" *** ");
        println!("{question}");

        for i in 0..answers.len() {
            println!("{}) {}", i+1, answers[i]);
        }

        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            println!("Не понимаю ответ");
            continue 'ask;
        } 

        let mut answer: Vec<usize> = answer
            .replace(",", " ")
            .replace("  ", " ")
            .trim()
            .split(" ")
            .map(|x| x.parse::<usize>().unwrap())
            .collect();

        for i in 0..answer.len() {
            if answer[i] as usize <= answers.len() && answer[i] > 0 {
                answer[i] -= 1;
            } else {
                continue 'ask;
            }
        }

        return answer;
    }
}



/// Выводит перечень тестов.
fn print_avaliable_tests() { 
    let request = Request {
        user: whoami::username(),
        test: "".to_string(),
        command: Command::GetAvaliableTests
    };

    match send_request(&request) {
        Ok(response) => {
            match response {
                Response::AvaliableTests { tests } => {
                    println!("Перечень доступных тестов:");
                    for (test, result) in tests {
                        println!("{test} {result}");
                    }
                },
                _ => eprintln!("Ошибка чтения списка тестов."),
            }
        },
        Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string())
    }
}


/// Осуществляет связь с сервером.
fn send_request(request: &Request)
    -> Result<Response, Box<dyn Error>> {

    let request = bincode::serialize(&request)?;
    let mut response = [0 as u8; 5000];
    
    let mut stream = TcpStream::connect(get_server_address())?;
    stream.write(&request)?;
    let n_bytes = stream.read(&mut response)?;
    
    let response = bincode::deserialize::<Response>(&response[..n_bytes])?;
    match response {
        Response::ServerError => {
            println!("Произошли технические шоколадки :(");
            println!("Организаторы уже в курсе, попробуйте вернуться к тестированию позже");
            std::process::exit(1);
        }

        resp => return Ok(resp)
    };
}

fn get_server_address() -> String {
    match std::env::var("SERVER_ADDRESS") {
        Ok(val) => {
            val
        }, 
        Err(_) => "127.0.0.1:65001".to_string()
    }
}
