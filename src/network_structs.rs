use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub user: String,
    pub command: Command 
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    GetAvaliableTests,
    StartTest { 
        test: String 
    },
    
    GetNextQuestion { 
        test: String,
        previos_answer: Vec<u8>,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
     AvaliableTests { tests: Vec<String> },
     StartTest { banner: String },
     GetNextQuestion { question: NextQuestion }, 
     NotAllowedUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NextQuestion {
    TheEnd { result: String },
    Question { 
        question: String,
        answers: Vec<String>
    }
}
