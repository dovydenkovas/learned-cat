pub mod exammanager;

/// Интерфейс взаимодействия Сервера и Экзаменатора.
pub trait Server {
    /// Взять запрос из очереди запроса.
    fn pop_request(&mut self) -> Option<lc_examiner::network::Request>;

    /// Отправить ответ на запрос.
    fn push_response(&mut self, response: lc_examiner::network::Response);
}
