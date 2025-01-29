use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use examiner::Examiner;
use learned_cat_interfaces::{
    network::{Request, Response},
    Config, Database, Server,
};

pub mod examiner;
pub mod server;

enum Tick {
    CollectCompletedTests,
    Request { request: Request },
}

struct ContollerChannel {
    tx: Sender<Tick>,
    rx: Arc<Mutex<Receiver<Response>>>,
}

pub struct ExaminerChannel {
    tx: Sender<Response>,
    rx: Receiver<Tick>,
}

pub struct Controller {
    srv: Arc<Mutex<dyn Server + Sync + Send>>,
    channel: ContollerChannel,
    examiner: Examiner,
}

impl Controller {
    pub fn new(
        config: Box<dyn Config>,
        db: Box<dyn Database>,
        srv: Arc<Mutex<dyn Server + Sync + Send>>,
    ) -> Controller {
        let (c_tx, e_rx) = mpsc::channel();
        let (e_tx, c_rx) = mpsc::channel();

        let channel = ExaminerChannel { tx: e_tx, rx: e_rx };
        let examiner = Examiner::new(config, db, channel);

        let channel = ContollerChannel {
            tx: c_tx,
            rx: Arc::new(Mutex::new(c_rx)),
        };
        let controller = Controller {
            srv,
            channel,
            examiner,
        };

        controller
    }

    /// Запусить сервер
    pub fn run(&mut self) {
        self.run_collector();
        self.run_mainloop();
        self.examiner.mainloop();
    }

    /// Пробуждаться каждые
    fn run_collector(&mut self) {
        let tx = self.channel.tx.clone();
        let _ = thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(2));
            tx.send(Tick::CollectCompletedTests).unwrap();
        });
    }

    /// Главный цикл обработки запросов.
    fn run_mainloop(&mut self) {
        let srv = Arc::clone(&self.srv);
        let rx = Arc::clone(&self.channel.rx);
        let tx = self.channel.tx.clone();
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
}
