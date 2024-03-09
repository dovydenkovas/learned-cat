use std::io::{Read, Write};
use std::error::Error;
use std::net::{TcpListener, TcpStream};

use clap::arg;

mod network;
use network::Request; 
mod presenter;
use presenter::Presenter;
mod model;


fn main() {
    let arguments = get_arguments();
    
    let root = std::path::Path::new("./sshtest");
    if std::env::set_current_dir(&root).is_err() {
        println!("Ошибка доступа к каталогу сервера {}. Проверьте, что каталог существует, и у процесса есть у нему доступ.", root.to_str().unwrap());
    }

    if arguments.subcommand_matches("init").is_some() {
        model::init::init_server();
    } else {
        let mut presenter: Presenter = Presenter::new();
        if arguments.subcommand_matches("start").is_some() {
            server_mainloop(&mut presenter);
        } else if let Some(arguments) = arguments.subcommand_matches("export-results") {
            presenter.export_results(arguments.get_one::<String>("filename").unwrap().to_string());
        } 
    }
}


fn get_arguments() -> clap::ArgMatches {
    clap::Command::new("sshtest-server")
        .version("0.1.0")
        .author("Aleksandr Dovydenkov. <asdovydenkov@gmail.com>")
        .about("Сервер автоматического тестирования в терминале. ")
        .subcommand(
            clap::Command::new("init")
                .about("создает файлы сервера: каталог /opt/sshtest и базу данных")
                .arg(arg!([postgres_credentials])),
        )
                
        .subcommand(
            clap::Command::new("start")
                .about("запускает сервер")
        )
 
        .subcommand(
            clap::Command::new("export-results")
    
            .about("экспортирует результаты тестирования в виде csv таблицы следующего формата: <student>:<test>:<result>")
                .arg(arg!([filename]).required(true)),
        )
        
        .get_matches()
}


/// Open listener and run mainloop
fn server_mainloop(presenter: &mut Presenter) {
    println!("* Открываю порт сервера: 127.0.0.1:65001");
    let listener = TcpListener::bind("127.0.0.1:65001").expect("Не могу открыть соединение");

    println!("* Запускаю главный цикл\n");
    loop {    
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    match handle_client(&mut stream, presenter) {
                        Ok(()) => (),
                        Err(err) => eprintln!("{err:?}")
                    }   
                },
                Err(err) => eprintln!("{err:?}")
            }
        }
    }
}


fn handle_client(stream: &mut TcpStream, presenter: &mut Presenter) -> Result<(), Box<dyn Error>> {
    let mut request = [0 as u8; 5000];
    let n_bytes = stream.read(&mut request)?;
    let request = bincode::deserialize::<Request>(&request[0..n_bytes])?;
    
    print!("[{}] {:?} -> ", chrono::Utc::now(),  request);
    let response = presenter.serve_connection(request);
    println!("{:?}", response);
    let response = bincode::serialize(&response)?;
    
    stream.write(&response)?;
    Ok(())
}


