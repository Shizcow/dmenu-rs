default:
	cargo build --release

run:
	cargo build && seq 1 100 | target/debug/dmenu $(ARGS)

reference:
	seq 1 100 | dmenu

debug:
	cargo build && seq 1 100 | valgrind --leak-check=full target/debug/dmenu

stest:
	cargo build && cargo run --bin stest

gdb:
	rust-gdb target/debug/dmenu-rs
