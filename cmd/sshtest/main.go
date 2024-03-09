package main


import (
	"flag"
	"fmt"
	"os"
	"path"
  "encoding/json"
)

type Question struct {
  Text string `json:"text"`
  Answers []string `json:"answers"`
}




type Response struct {
  IsCorrect `json:"is_correct"`
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


func runTest(testname string) {
  if makeRequest() 
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
