pub type ModelResult<T> = std::result::Result<T, ModelError>;

/// Errors that can be generated in daemon
#[derive(Debug, Clone)]
pub enum ModelError {
    TestNotExist(String),
    TestIsDone,
    ResultNotExist(String),
    UserNotAllowed,
    VariantNotExist(String),
    TestIsOpened(String, String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let res = match &self {
            ModelError::TestIsDone => "Тест завершен.".to_string(),
            ModelError::TestNotExist(testname) => format!("Теста '{testname}' не существует."),
            ModelError::ResultNotExist(result) => format!("Результата '{result}' не существует."),
            ModelError::UserNotAllowed => "Пользователь не имеет доступа к операции.".to_string(),
            ModelError::VariantNotExist(variant) => format!("Варианта '{variant}' не существует."),
            ModelError::TestIsOpened(user, test) => {
                format!("Тест {test} уже выполняется пользователем {user}.")
            }
        };
        write!(f, "{res}")
    }
}
