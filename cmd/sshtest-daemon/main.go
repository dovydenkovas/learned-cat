package main

import (
	"bufio"
	"encoding/json"
	"errors"
	"log"
	"net"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"time"

	"github.com/BurntSushi/toml"
)

////////////////
// Structures //
////////////////


type Question struct {
  Question string `json:"question"`
  Answers []string `json:"answers"`
  trueAnswers []int
}


type Test struct {
  questions []Question
}


type Request struct {
    User    string `json:"user"`
    Command string `json:"command"`
}


type ConfigTest struct {
  Name string `toml:"name"`
  ValidUsers []string `toml:"valid_users"`
  Duration int `toml:"duration"`
  ShowResults bool `toml:"show_results"`
  NumberOfAttempts int `toml:"number_of_attempts"`
}


type Config struct {
  TestPath string `toml:"test_path"`
  ResultPath string `toml:"result_path"`
  Tests []ConfigTest `toml:"test"`
}



///////////////////////
// Initial functions //
///////////////////////


func parseTest(path string) Test {
  // TODO: parse header 
  // TODO: refactor 
  var test Test

  f, err := os.Open(path)
  if errors.Is(err, os.ErrNotExist) {
    log.Fatal("Ошибка чтения теста: ", err)
    } else {
    defer f.Close()
    scanner := bufio.NewScanner(f)
    
    var is_question = true 
    for scanner.Scan() {
      var text = strings.Trim(scanner.Text(), " \t")

      if len(text) > 0 {
        if text[0] == '#' { // New question
          var question Question
          question.Question = text[1:]
          test.questions = append(test.questions, question)
          is_question = true   
        } else 

        if text[0] == '+' { // New true answer
          var answer = text[1:]
          var true_answer_id = len(test.questions[len(test.questions)-1].Answers)
          test.questions[len(test.questions)-1].Answers = append(test.questions[len(test.questions)-1].Answers, answer)
          test.questions[len(test.questions)-1].trueAnswers = append(test.questions[len(test.questions)-1].trueAnswers, true_answer_id)  
          is_question = false 
        } else 

        if text[0] == '*' || text[0] == '-' { // New false answer
          var answer = text[1:]
          test.questions[len(test.questions)-1].Answers = append(test.questions[len(test.questions)-1].Answers, answer)
          is_question = false 
        } else 

        if is_question { // next string of question 
          id := len(test.questions)-1
          test.questions[id].Question += "\n" + text  

        } else { // next string of answer
           q_id := len(test.questions)-1
           a_id := len(test.questions[q_id].Answers) -1 
           test.questions[q_id].Answers[a_id] += "\n" + text  
        }
      }
    }
    

    if err := scanner.Err(); err != nil {
      log.Fatal(err)
    }
  }
  return test 
}


func parseConfig(path string) Config {
  f, err1 := os.Open(path)
  if err1 != nil {
    log.Fatal(err1)
  }
  
  var buff = make([]byte, 10000) 
  l, _ := f.Read(buff) 
  tomlData := string(buff[:l])
  
  var conf Config
  if _, err := toml.Decode(tomlData, &conf); err != nil {
    log.Fatal(err)
  }

  return conf 
}



///////////////////////////
// Serve users functions //
///////////////////////////


type ActiveTests struct { // Use hashmap of [user@test, ActiveTests]
  UserName string 
  TestName string 
  BeginTimestamp time.Time
  Variant [][]int 
  Answers [][]int 
}


func checkDone(user *string, test *string, config *Config) bool {
  return false
}


func startTest(user *string, test *string, config *Config, activeTests *[]ActiveTests) {
  
}


func nextQuestion(user *string, test *string, config *Config, tests *[]Test, activeTests *[]ActiveTests) Question {

}


func addAnswer(user *string, test *string, config *Config, tests *[]Test, activeTests *[]ActiveTests) {

}


func endTest(user *string, test *string, config *Config, tests *[]Test, activeTests *[]ActiveTests) string {

}


func clearActiveTests(config *Config, tests *[]Test, activeTests *[]ActiveTests) {

}


///////////////////////
// Network functions //
///////////////////////


func handleConnection(c net.Conn, tests *[]Test, config *Config) {
  d := json.NewDecoder(c)
  var r Request
  err:=d.Decode(&r)

  if err != nil {
    log.Println(r, err)
  }
  
  log.Println("Request: ", r)

  if _, f := slices.BinarySearch(config.Tests[0].ValidUsers, r.User); !f {
    c.Close()
    return 
  }

  switch r.Command {
  case "check_done":
    if checkDone(&r.User) {
      c.Write([]byte("true"))
    } else {
      c.Write([]byte("false"))
    }
    
  
  case "get_banner":
    c.Write([]byte("\"Описание теста\""))
  
  case "get_variant":
    cd := json.NewEncoder(c)
    cd.Encode(tests[0].questions)

  case "check_answer":

  case "end_test":

  default:
    log.Println("Request: ", r)
  }
  
  c.Close()
}


func netMainloop(tests *[]Test, config *Config) {
    ln, err := net.Listen("tcp", ":65431")

    if err != nil {
      log.Fatal(err)
    }

    for {
      c, err := ln.Accept() 
      if err != nil {
        log.Print(err)
      } else {
        go handleConnection(c, tests, config) 
      }
    }
}




//////////
// Main //
//////////


func main() {
  log.Println("Загружаю конфиг")
  var config = parseConfig("config.toml")

  var tests []Test
  
  for _, config_test := range config.Tests {
    path := filepath.Join(config.TestPath, config_test.Name)
    log.Print("Загружаю: ", path)
    var test = parseTest(path)
    tests = append(tests, test)
    slices.Sort(config_test.ValidUsers)
    log.Println(" ", len(test.questions), "вопросов.")
  }
    
  log.Println("Запуск сервера")
  netMainloop(&tests, &config)
}
