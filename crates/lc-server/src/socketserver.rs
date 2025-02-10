use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use lc_examiner::network;
use lc_exammanager::Server;
use log::{debug, info};

pub struct SocketServer {
    listener: TcpListener,
    stream: Option<TcpStream>,
}

impl SocketServer {
    pub fn new(address: String) -> SocketServer {
        info!("Открываю порт сервера: {}", address);
        let listener = TcpListener::bind(address).expect("Не могу открыть соединение");

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
                let request =
                    bincode::deserialize::<network::Request>(&request[0..n_bytes]).unwrap();

                self.stream = Some(stream);
                debug!("{request:?}");
                Some(request)
            }
            Err(_) => None,
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

#[cfg(test)]
mod tests {
    use learned_cat_interfaces::schema::Answer;
    use network::Response;
    use std::{
        error::Error,
        thread::{self, sleep},
        time::Duration,
    };

    use super::*;
    use ntest::timeout;

    /// Осуществляет связь с сервером.
    fn send_request(request: &Request, listen: String) -> Result<Response, Box<dyn Error>> {
        let request = bincode::serialize(&request)?;
        let mut response = [0 as u8; 1_000_000];

        let mut stream = TcpStream::connect(listen)?;
        stream.write(&request)?;
        let n_bytes = stream.read(&mut response)?;

        let response = bincode::deserialize::<Response>(&response[..n_bytes])?;
        match response {
            Response::ServerError => {
                std::process::exit(1);
            }

            resp => return Ok(resp),
        };
    }

    #[test]
    fn network_single_request() {
        let mut srv = SocketServer::new("127.0.0.1:8888".to_string());

        thread::spawn(|| {
            let req = Request::new("user", "test", network::Command::StartTest);
            let resp = send_request(&req, "127.0.0.1:8888".to_string()).unwrap();
            assert_eq!(resp, Response::Ok);
        });

        sleep(Duration::from_millis(1));
        let reqq = srv.pop_request().unwrap();
        assert_eq!(
            reqq,
            Request::new("user", "test", network::Command::StartTest)
        );
        srv.push_response(Response::Ok);
    }

    #[test]
    #[timeout(100)]
    fn network_multi_request() {
        let mut srv = SocketServer::new("127.0.0.1:8889".to_string());

        thread::spawn(|| {
            for i in 1..102 {
                let req = Request::new(
                    "user",
                    "test",
                    network::Command::PutAnswer {
                        answer: Answer::new(vec![i, 2 * i, i * i]),
                    },
                );
                let resp = send_request(&req, "127.0.0.1:8889".to_string()).unwrap();
                assert_eq!(
                    resp,
                    Response::NextQuestion {
                        question: "oops!\ntext.".to_string(),
                        answers: vec!["A".to_string(), "B".to_string()]
                    }
                );
            }
        });

        sleep(Duration::from_millis(1));
        let mut i = 0;
        let mut reqq = srv.pop_request();
        while reqq.is_some() && i < 100 {
            i += 1;
            let req = Request::new(
                "user",
                "test",
                network::Command::PutAnswer {
                    answer: Answer::new(vec![i, 2 * i, i * i]),
                },
            );

            assert_eq!(reqq.unwrap(), req);
            let resp = Response::NextQuestion {
                question: "oops!\ntext.".to_string(),
                answers: vec!["A".to_string(), "B".to_string()],
            };
            srv.push_response(resp);
            sleep(Duration::from_micros(500));
            reqq = srv.pop_request();
        }

        assert_eq!(i, 100);
    }
}
