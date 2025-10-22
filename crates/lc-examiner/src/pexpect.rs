use std::process::exit;

pub trait Pexpect<T> {
    fn pexpect<S: std::fmt::Display>(self, s: S) -> T;
}

impl<T, E> Pexpect<T> for Result<T, E> {
    fn pexpect<S: std::fmt::Display>(self, msg: S) -> T {
        match self {
            Ok(t) => t,
            Err(_) => {
                eprintln!("{}", msg);
                exit(1);
            }
        }
    }
}
