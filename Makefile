all: sshtest sshtest-daemon 


sshtest: 
	go build -o sshtest ./cmd/sshtest/main.go 

sshtest-daemon:
	go build -o sshtest-daemon ./cmd/sshtest-daemon/main.go 

