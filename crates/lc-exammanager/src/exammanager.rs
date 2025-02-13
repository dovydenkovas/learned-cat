use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use log::{debug, error};

use crate::Server;

use lc_examiner::{
    examiner::Examiner,
    network::{Command, Request, Response},
};

enum Tick {
    CollectCompletedTests,
    Request { request: Request },
}

struct ContollerChannel {
    tx: Sender<Tick>,
    rx: Arc<Mutex<Receiver<Response>>>,
}

struct ExaminerChannel {
    tx: Sender<Response>,
    rx: Receiver<Tick>,
}

pub struct ExamManager {
    srv: Arc<Mutex<dyn Server + Sync + Send>>,
    controller_channel: ContollerChannel,
    examiner_channel: ExaminerChannel,
    examiner: Examiner,
}

impl ExamManager {
    pub fn new(examiner: Examiner, srv: Arc<Mutex<dyn Server + Sync + Send>>) -> ExamManager {
        let (c_tx, e_rx) = mpsc::channel();
        let (e_tx, c_rx) = mpsc::channel();

        let examiner_channel = ExaminerChannel { tx: e_tx, rx: e_rx };
        let controller_channel = ContollerChannel {
            tx: c_tx,
            rx: Arc::new(Mutex::new(c_rx)),
        };

        ExamManager {
            srv,
            controller_channel,
            examiner_channel,
            examiner,
        }
    }

    /// Запусить сервер
    pub fn run(&mut self) {
        self.run_collector();
        self.run_mainloop();
        self.examiner_mainloop();
    }

    /// Пробуждаться каждые 2 секунды
    fn run_collector(&mut self) {
        debug!("Запускаю проверку ограничения времени тестирования.");
        let tx = self.controller_channel.tx.clone();
        let _ = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(2));
            tx.send(Tick::CollectCompletedTests).unwrap();
        });
    }

    /// Обработка входящих запросов.
    fn run_mainloop(&mut self) {
        debug!("Запускаю обработчик запросов.");
        let srv = Arc::clone(&self.srv);
        let rx = Arc::clone(&self.controller_channel.rx);
        let tx = self.controller_channel.tx.clone();
        let _ = thread::spawn(move || {
            let mut srv = srv.lock().unwrap();
            let rx = rx.lock().unwrap();
            loop {
                let request = srv.pop_request().unwrap();
                tx.send(Tick::Request { request }).unwrap();
                srv.push_response(rx.recv().unwrap());
            }
        });
    }

    /// Главный цикл обработки задач.
    fn examiner_mainloop(&mut self) {
        debug!("Запускаю главный цикл обработки задач.");
        loop {
            match self.examiner_channel.rx.recv() {
                Ok(tick) => match tick {
                    Tick::CollectCompletedTests => self.examiner.variant_collector(),
                    Tick::Request { request } => {
                        let responce = self.serve_request(request);
                        self.examiner_channel.tx.send(responce).unwrap();
                    }
                },
                Err(err) => error!("Ошибка обработки запроса: {:?}.", err),
            }
        }
    }

    /// Обработать запрос клиента.
    fn serve_request(&mut self, request: Request) -> Response {
        match request.command {
            Command::StartTest => self
                .examiner
                .banner_to_start_test(&request.user, &request.test),
            Command::GetNextQuestion => self.examiner.next_question(&request.user, &request.test),
            Command::GetAvaliableTests => self.examiner.avaliable_tests(&request.user),
            Command::PutAnswer { answer } => {
                self.examiner
                    .put_answer(&request.user, &request.test, &answer)
            }
        }
    }
}
