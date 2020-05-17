default:
	cargo build --release

run:
	cargo build && echo -e 'ABC\nabc\nbCa\nbC' | target/debug/dmenu-rs $(ARGS)

reference:
	echo -e 'ABC\nabc\nbCa\nbC' | dmenu -p AA
# seq 1 100 | dmenu -w $(shell xdotool getmouselocation --shell | grep -Po '(?<=WINDOW\=).*')

debug:
	cargo build && seq 1 100 | valgrind --leak-check=full target/debug/dmenu-rs

gdb:
	rust-gdb target/debug/dmenu-rs
