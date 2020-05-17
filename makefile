default:
	cargo build --release

run:
	cargo build && echo -e 'ABC\nabc\nbCa\nbC0' | target/debug/dmenu-rs $(ARGS)

reference:
	echo -e 'ABC\nabc\nbCa\nbC0' | dmenu

debug:
	cargo build && seq 1 100 | valgrind --leak-check=full target/debug/dmenu-rs -w 69206335

gdb:
	rust-gdb target/debug/dmenu-rs
