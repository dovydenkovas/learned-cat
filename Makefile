all:
	cargo	build --release
	cp target/release/sshtest sshtest
	cp target/release/sshtest-daemon sshtest-daemon
