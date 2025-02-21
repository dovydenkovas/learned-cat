/// Клиентское приложение программы тестирования.
/// Отправляет запросы на сервер и предоставляет пользовательский интерфейс.
use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;

use rustyline::DefaultEditor;
use whoami;

use lc_examiner::{
    network::{Command, Marks, Request, Response},
    schema::Answer,
};

/// Парсит аргументы и запускает соответствующее действие.
fn main() {
    match std::env::args().nth(1) {
        Some(v) => match v.as_str() {
            "-l" | "--list" => print_avaliable_tests(),
            "-h" | "--help" => print_help(),
            "-V" | "--version" => println!("learned-cat 0.2.0"),
            test => start_test(test.to_string()),
        },
        None => print_help(),
    }
}

fn print_help() {
    println!(
        r#"Использование:
    - Для запуска теста:
        learned-cat [НАЗВАНИЕ_ТЕСТА]

    - Для получения информации:
        learned-cat [ПАРАМЕТР]


ПАРАМЕТРЫ:
    -l, --list     Отобразить доступные тесты
    -h, --help     Показать эту справку
    -V, --version  Отобразить номер версии

Об ошибках сообщайте asdovydenkov@yandex.ru
Последняя версия доступна по адресу: https://github.com/dovydenkovas/learned-cat"#
    );
}

/// Обслуживает процесс тестирования.
fn start_test(test_name: String) {
    let request = Request::new(whoami::username(), test_name.clone(), Command::StartTest);

    match send_request(&request) {
        Ok(response) => match response {
            Response::TestStarted { banner } => {
                println!("{banner}");
                println!("Вы готовы начать тестирование? (y/n)");
                if ask_yes() {
                    run_test(test_name, None);
                }
            }

            Response::NextQuestion { question, answers } => run_test(
                test_name,
                Some(Response::NextQuestion { question, answers }),
            ),

            Response::End { marks } => {
                print!("Тест завершён. Ваш результат: ");
                print_marks(marks);
            }

            _ => print_help(),
        },
        Err(_) => eprintln!("Ошибка связи с сервером. Пожалуйста, повторите попытку позже."),
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
                    Command::PutAnswer {
                        answer: Answer::new(answers),
                    },
                );

                match send_request(&put_answer_request) {
                    Ok(Response::End { marks }) => {
                        print!("Тест завершён. Ваш результат: ");
                        print_marks(marks);
                        break;
                    }

                    Ok(Response::NextQuestion { question, answers }) => {
                        next_question = Some(Response::NextQuestion { question, answers })
                    }

                    _ => (),
                }
            }

            Ok(Response::End { marks }) => {
                print!("Тест завершён. Ваш результат: ");
                print_marks(marks);
                break;
            }

            _ => (),
        }
    }
}

/// Вывод результата
fn print_marks(marks: Marks) {
    match marks {
        Marks::Marks { marks } => {
            for mark in marks {
                print!("{mark:.2} ");
            }
            println!("");
        }
        Marks::Done => {
            println!("Тест завершён.");
        }
        Marks::Empty => {
            println!("");
        }
    }
}

/// Задает вопрос
fn ask_question(question: String, answers: Vec<String>) -> Vec<usize> {
    println!("");
    println!("        ***");
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
        Err(_) => eprintln!("Ошибка связи с сервером. Пожалуйста, повторите попытку позже."),
    }
}

/// Вывод таблицы ключ-значение
fn print_table(values: Vec<(String, Marks)>) {
    let mut max_first = 0;
    for first in &values {
        max_first = std::cmp::max(max_first, first.0.len());
    }

    println!("{:>max_first$}   Ваш результат", "Тест");
    for first in &values {
        print!("{:>max_first$} ", first.0);
        print_marks(first.1.clone());
    }
}

/// Осуществляет связь с сервером.
fn send_request(request: &Request) -> Result<Response, Box<dyn Error>> {
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
