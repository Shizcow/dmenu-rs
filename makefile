default:
	cargo build --release

run:
	cargo build && seq 1 100 | target/debug/dmenu-rs $(ARGS)

reference:
	seq 1 100 | dmenu

debug:
	cargo build && seq 1 100 | valgrind --leak-check=full target/debug/dmenu-rs

gdb:
	rust-gdb target/debug/dmenu-rs
