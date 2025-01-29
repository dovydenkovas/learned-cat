#![allow(unused)]
use std::io::prelude::*;
use std::net::TcpStream;
/// Клиентское приложение программы тестирования.
/// Отправляет запросы на сервер и предоставляет пользовательский интерфейс.
use std::{error::Error, thread};

use whoami;

use learned_cat_interfaces::{
    network::{Command, Request, Response},
    schema::Answer,
};

/// Парсит аргументы и запускает соответствующее действие.
fn main() {
    test_tests();
    // test_avaliable_tests();
}

fn test_avaliable_tests() {
    let start = chrono::Local::now();
    let mut n = 0;
    for _ in 0..1000 {
        let request = Request::new(
            whoami::username(),
            "".to_string(),
            Command::GetAvaliableTests,
        );

        match send_request(&request, &mut n) {
            Ok(response) => match response {
                Response::AvaliableTests { tests } => {}
                _ => eprintln!("Ошибка чтения списка тестов."),
            },
            Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string()),
        }
    }

    let diff = chrono::Local::now() - start;
    println!("Результаты посмотрены за {}", diff);
    println!(
        "Среднее время выполнения запроса (мс): {}",
        diff.num_milliseconds() as f64 / n as f64
    );
    println!(
        "Количество запросов в секунду: {}",
        1e9 * n as f64 / diff.num_nanoseconds().unwrap() as f64
    );
}

fn test_tests() {
    let start = chrono::Local::now();
    let mut n = 1000;
    for i in 0..n {
        let k = i.clone();
        //thread::spawn(move || {
        //    let mut n = 0;
        start_test("algo".to_string(), &mut n, k % 250);
        //});
    }

    let diff = chrono::Local::now() - start;
    println!("\n");
    println!("Тесты выполнены за {}", diff);
    println!(
        "Среднее время выполнения запроса (мс): {}",
        diff.num_milliseconds() as f64 / n as f64
    );
    println!(
        "Количество запросов в секунду: {}",
        1e9 * 12.0 * n as f64 / diff.num_nanoseconds().unwrap() as f64
    );
    println!("Всего выполнено запросов: {n}\n");
}

fn start_test(test_name: String, n: &mut usize, k: usize) {
    let request = Request::new(format!("user{k}"), test_name.clone(), Command::StartTest);

    match send_request(&request, n) {
        Ok(response) => match response {
            Response::TestStarted { banner } => {
                run_test(test_name, None, n, k);
            }

            _ => eprintln!("Сервер не ответил"),
        },
        Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string()),
    }
}

fn run_test(test_name: String, next_question: Option<Response>, n: &mut usize, k: usize) {
    let next_question_request = Request::new(
        format!("user{k}"),
        test_name.clone(),
        Command::GetNextQuestion,
    );

    let mut next_question = next_question;
    loop {
        let response = match next_question {
            Some(q) => {
                next_question = None;
                Ok(q)
            }
            None => send_request(&next_question_request, n),
        };

        match response {
            Ok(Response::NextQuestion { question, answers }) => {
                let answers = vec![0];
                let put_answer_request = Request::new(
                    format!("user{k}"),
                    test_name.clone(),
                    Command::PutAnswer {
                        answer: Answer::new(answers),
                    },
                );

                match send_request(&put_answer_request, n) {
                    Ok(Response::End { result }) => {
                        println!("{:?}", result);
                        break;
                    }

                    Ok(Response::NextQuestion { question, answers }) => {
                        next_question = Some(Response::NextQuestion { question, answers })
                    }

                    _ => (),
                }
            }

            Ok(Response::End { result }) => {
                break;
            }

            _ => (),
        }
    }
}

/// Осуществляет связь с сервером.
fn send_request(request: &Request, n: &mut usize) -> Result<Response, Box<dyn Error>> {
    *n += 1;
    let request = bincode::serialize(&request)?;
    let mut response = [0 as u8; 1_000_000];

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

        resp => return Ok(resp),
    };
}

fn get_server_address() -> String {
    match std::env::var("SERVER_ADDRESS") {
        Ok(val) => val,
        Err(_) => "127.0.0.1:8080".to_string(),
    }
}
