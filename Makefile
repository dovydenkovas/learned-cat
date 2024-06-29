all:
	cargo	build --release
	cp target/release/learned-cat learned-cat
	cp target/release/learned-cat-daemon learned-cat-daemon
