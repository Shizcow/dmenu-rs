default:
	cargo build --release

run:
	cargo build && seq 1 100 | target/debug/dmenu-rs

reference:
	 seq 1 100 | dmenu -p "Prompt"

debug:
	cargo build && seq 1 100 | valgrind --leak-check=full target/debug/dmenu-rs

gdb:
	rust-gdb target/debug/dmenu-rs
