all:
	cargo	build --release
	cp target/release/learned-cat learned-cat
	cp target/release/learned-cat-daemon learned-cat-daemon

clean:
	rm -f learned-cat learned-cat-daemon

purge: clean
	rm -rf target
