default:
	cargo build --release

run:
	cargo build && echo -e '1\n2\n3' | target/debug/dmenu-rs


debug:
	cargo build && echo -e '1\n2\n3' | valgrind -v target/debug/dmenu-rs

gdb:
	rust-gdb target/debug/dmenu-rs
