/// Структуры сетевой коммуникации между сервером и клиеном
///
/// Порядок обмена.
/// Выполнение теста:
/// -> Request:StartTest
/// <- Response:TestStarted (Проверяются учетные данные, выдается приветствие)
/// (Или <- Response:End (Строка заключения) если тест уже пройден)
/// -> Request:GetNextQuestion
/// <- Response:NextQuestion (Фиксируется время начала теста, выдается вопрос)
/// -> Response:PutAnswer
/// <- Response:Ok (Подтверждение принятия вопроса)
/// -> Request:NextQuestion
/// <- Response:End (Выдается строка заключения)
///
/// Получение списка тестов:
/// -> Request:GetAvaliableTests,
/// <- Response:AvaliableTests (список тестов, доступных пользователю с указанием
///                             результатов, если есть)
///
/// Обработка ошибок:
/// -> Request (любой запрос)
/// <- Response:AccessDenied - Пользователя нет в белом списке
///
/// -> Request (любой запрос)
/// <- Response:ServerError - ошибка на стороне сервера
///
/// -> Request (любой запрос)
/// <- Response:ResponseError - некорректный запрос
use serde::{Deserialize, Serialize};

use crate::schema::Answer;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub user: String,
    pub test: String,
    pub command: Command,
}

impl Request {
    pub fn new<S: AsRef<str>>(user: S, test: S, command: Command) -> Request {
        Request {
            user: user.as_ref().to_string(),
            test: test.as_ref().to_string(),
            command,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Command {
    GetAvaliableTests,
    StartTest,
    GetNextQuestion,
    PutAnswer { answer: Answer },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Marks {
    Marks { marks: Vec<f32> },
    Done,
    Empty,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    AvaliableTests {
        tests: Vec<(String, Marks)>,
    }, // Название теста и результат
    TestStarted {
        banner: String,
    },
    NextQuestion {
        question: String,
        answers: Vec<String>,
    },
    Ok,
    End {
        marks: Marks,
    },
    NotAllowedUser,
    ServerError,
    ResponseError,
}
