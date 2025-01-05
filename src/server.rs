use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use learned_cat_interfaces::{
    network::{self, Request},
    Server,
};

pub struct SocketServer {
    listener: TcpListener,
    stream: Option<TcpStream>,
}

impl SocketServer {
    pub fn new(address: String) -> SocketServer {
        println!("* Открываю порт сервера: {}", address);
        let listener = TcpListener::bind(address).expect("Не могу открыть соединение");
        let _ = listener.set_nonblocking(true);

        SocketServer {
            listener,
            stream: None,
        }
    }
}

impl Server for SocketServer {
    /// Взять запрос из очереди запроса.
    fn pop_request(&mut self) -> Option<network::Request> {
        match self.listener.accept() {
            Ok((mut stream, _)) => {
                let mut request = [0 as u8; 5000];
                let n_bytes = stream.read(&mut request).unwrap();
                let request = bincode::deserialize::<Request>(&request[0..n_bytes]).unwrap();

                self.stream = Some(stream);
                Some(request)
            }
            Err(_) => return None,
        }
    }

    /// Отправить ответ на запрос.
    fn push_response(&mut self, response: network::Response) {
        let response = bincode::serialize(&response).unwrap();
        if self.stream.is_some() {
            let _ = self.stream.as_mut().unwrap().write(&response);
        }
    }
}
