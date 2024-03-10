all:
	cargo	build --release
	mv target/release/sshtest sshtest
	mv target/release/sshtest-daemon sshtest-daemon
