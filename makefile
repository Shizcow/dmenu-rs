default:
	cargo build --release

test: 	default
	seq 1 100 | target/release/dmenu $(ARGS)

clean:
	rm -rf vgcore* massif* target

VERSION:=$(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)
dist:	
	mkdir -p dmenu-$(VERSION)
	cp -r LICENSE README.md README makefile build.rs Cargo.toml src dmenu-$(VERSION)
	tar -cf dmenu-$(VERSION).tar dmenu-$(VERSION)
	gzip dmenu-$(VERSION).tar
	rm -rf dmenu-$(VERSION)
