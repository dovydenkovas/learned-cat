all: sshtest sshtest-daemon 


sshtest: 
	go build ./cmd/sshtest/main.go

sshtest-daemon:
	go build ./cmd/sshtest-daemon/main.go

