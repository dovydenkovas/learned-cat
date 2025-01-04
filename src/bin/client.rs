/// Клиентское приложение программы тестирования.
/// Отправляет запросы на сервер и предоставляет пользовательский интерфейс.
use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;

use clap::Parser;
use rustyline::DefaultEditor;
use whoami;

use learned_cat::network::{Command, Request, Response};

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
    let request = Request::new(whoami::username(), test_name.clone(), Command::StartTest);

    match send_request(&request) {
        Ok(response) => match response {
            Response::TestStarted { banner } => {
                println!("{banner}");
                println!("Вы готовы начать тестирование? (Введите <да> или <нет>)");
                if ask_yes() {
                    run_test(test_name, None);
                }
            }

            Response::NextQuestion { question, answers } => run_test(
                test_name,
                Some(Response::NextQuestion { question, answers }),
            ),

            Response::End { result } => {
                println!("Тест завершен. Ваш результат: {:?}", result);
            }

            _ => eprintln!("Теста не существует или доступ к нему закрыт."),
        },
        Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string()),
    }
}

fn ask_yes() -> bool {
    loop {
        let mut rl = match DefaultEditor::new() {
            Ok(v) => v,
            _ => continue,
        };

        let s = match rl.readline(">>> ") {
            Ok(v) => v,
            Err(rustyline::error::ReadlineError::Eof) => std::process::exit(0),
            _ => continue,
        };
        match s.trim().to_lowercase().as_str() {
            "yes" | "y" | "да" | "д" => return true,
            "no" | "n" | "нет" | "н" => return false,
            _ => (),
        }
    }
}

fn run_test(test_name: String, next_question: Option<Response>) {
    let next_question_request = Request::new(
        whoami::username(),
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
            None => send_request(&next_question_request),
        };

        match response {
            Ok(Response::NextQuestion { question, answers }) => {
                let answers = ask_question(question, answers);
                let put_answer_request = Request::new(
                    whoami::username(),
                    test_name.clone(),
                    Command::PutAnswer { answer: answers },
                );

                match send_request(&put_answer_request) {
                    Ok(Response::End { result }) => {
                        println!("Тест завершен. Ваш результат: {:?}", result);
                        break;
                    }

                    _ => (),
                }
            }

            Ok(Response::End { result }) => {
                println!("Тест завершен. Ваш результат: {:?}", result);
                break;
            }

            _ => (),
        }
    }
}

/// Задает вопрос
fn ask_question(question: String, answers: Vec<String>) -> Vec<usize> {
    println!("");
    println!("{:>len$}", "***", len = question.len() / 2 - 1);
    println!("{question}");
    for i in 0..answers.len() {
        println!("{}) {}", i + 1, answers[i]);
    }

    'ask: loop {
        let answer = ask_string();
        let mut answer: Vec<usize> = answer
            .replace(",", " ")
            .replace("  ", " ")
            .trim()
            .split(" ")
            .map(|x| match x.parse::<usize>() {
                Ok(v) => v,
                _ => 100000000,
            })
            .collect();

        for i in 0..answer.len() {
            if answer[i] as usize <= answers.len() && answer[i] > 0 {
                answer[i] -= 1;
            } else {
                println!("Пожалуйста, введите номера правильных ответов через пробел.");
                continue 'ask;
            }
        }

        return answer;
    }
}

fn ask_string() -> String {
    loop {
        let mut rl = match DefaultEditor::new() {
            Ok(v) => v,
            _ => continue,
        };

        match rl.readline(">>> ") {
            Ok(v) => return v,
            Err(rustyline::error::ReadlineError::Eof) => std::process::exit(0),
            _ => continue,
        };
    }
}

/// Выводит перечень тестов.
fn print_avaliable_tests() {
    let request = Request::new(
        whoami::username(),
        "".to_string(),
        Command::GetAvaliableTests,
    );

    match send_request(&request) {
        Ok(response) => match response {
            Response::AvaliableTests { tests } => {
                println!("Перечень доступных тестов:");
                print_table(tests);
            }
            _ => eprintln!("Ошибка чтения списка тестов."),
        },
        Err(err) => eprintln!("Ошибка связи с сервером: {}", err.to_string()),
    }
}

/// Вывод таблицы ключ-значение
fn print_table(values: Vec<(String, String)>) {
    let mut max_first = 0;
    for (first, _) in &values {
        max_first = std::cmp::max(max_first, first.len());
    }

    println!("{:>max_first$}   Ваш результат", "Тест");
    for (first, second) in &values {
        println!("{first:>max_first$}   {second:>6}");
    }
}

/// Осуществляет связь с сервером.
fn send_request(request: &Request) -> Result<Response, Box<dyn Error>> {
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

        resp => return Ok(resp),
    };
}

fn get_server_address() -> String {
    match std::env::var("SERVER_ADDRESS") {
        Ok(val) => val,
        Err(_) => "127.0.0.1:65001".to_string(),
    }
}
