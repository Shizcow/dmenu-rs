# dmenu-rs - dynamic menu
# See LICENSE file for copyright and license details.

include config.mk

VERSION:=$(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)


default:
ifeq ($(XINERAMA),true)
	cargo build --release --features "Xinerama"
else
	cargo build --release
endif

test: 	default
	seq 1 100 | target/release/dmenu $(ARGS)

clean:
	rm -rf vgcore* massif* target

dist:	default
	mkdir -p dmenu-$(VERSION)
	cp -r LICENSE README.md README makefile build.rs Cargo.toml src dmenu-$(VERSION)
	tar -cf dmenu-$(VERSION).tar dmenu-$(VERSION)
	gzip dmenu-$(VERSION).tar
	rm -rf dmenu-$(VERSION)
