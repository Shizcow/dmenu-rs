default:
	cargo build --release

run:
	RUSTFLAGS="-C debuginfo=2" cargo build && echo -e '1\n2\n3' | target/debug/dmenu-rs


debug:
	RUSTFLAGS="-C debuginfo=2" cargo build && echo -e '1\n2\n3' | valgrind -v target/debug/dmenu-rs

gdb:
	rust-gdb target/debug/dmenu-rs
