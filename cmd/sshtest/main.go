package main


import (
	"flag"
	"fmt"
	"os"
	"path"
//  "encoding/json"
)

type Question struct {
  Text string `json:"text"`
  Answers []string `json:"answers"`
}




type Response struct {
  IsCorrect bool `json:"is_correct"`
  IsTestDone bool `json:"is_test_done"`
  NextQuestion Question `json:"question"`
  Prompt string `json:"prompt"`  
}



func callServer(request string) {
//  ip := "127.0.0.1"
//  port := "65431"
  fmt.Println(request)
}


func makeRequest(request string) []string{
  return []string{"linux", "python"}
}


func getNextQuestion(testname string) (Question, error) {
  return Question {
    Text: "to be or not to be?",
    Answers: []string{"to be", "no to be"},
  }, nil
}

func initTest(testname string) bool {
  return true 
}


func playQuestion(question Question) []int {
  fmt.Println()
  fmt.Println(question.Text)
  for i, text := range question.Answers {
    fmt.Printf("%d) %s\n", i+1, text);
  }
  
  fmt.Print(">>> ")

  var answer int 
  fmt.Scanf("%d", &answer)

  return []int{answer}
}


func sendAnswer(testname string, answer []int) {

}


func runTest(testname string) {
  if initTest(testname) {
    for question, err := getNextQuestion(testname); err == nil; {
      answer := playQuestion(question);
      sendAnswer(testname, answer);
    }
  }
}


func showTests() {
  testsnames := makeRequest("list") 
  fmt.Println("Доступные тесты:")
  for _, testname := range testsnames {
    fmt.Printf("- %s\n", testname)
  }
}


func main() {
  isShowTests := flag.Bool("list", false, "Показать список доступных тестов")

  flag.Parse()

  if (*isShowTests) {
    showTests()
  } else if len(os.Args) == 2 {
      runTest(os.Args[1])
  } else {
    fmt.Printf("USAGE: %s <название теста>\n", path.Base(os.Args[0]))
  }
}
