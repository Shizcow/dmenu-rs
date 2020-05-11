default:
	cargo build --release

run:
	cargo build && seq 1 100 | target/debug/dmenu-rs

reference:
	 seq 1 100 | dmenu -p 1234X

debug:
	cargo build && seq 1 100 | valgrind -v target/debug/dmenu-rs

gdb:
	rust-gdb target/debug/dmenu-rs
