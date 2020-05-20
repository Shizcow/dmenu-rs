# dmenu-rs - dynamic menu
# See LICENSE file for copyright and license details.

include config.mk

VERSION:=$(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)

ifeq ($(XINERAMA),true)
	XINERAMA_FLAGS = --features "Xinerama"
endif


export CC
export CFLAGS
default:
	cargo build --release $(XINERAMA_FLAGS)

test:
	seq 1 100 | cargo run --release $(XINERAMA_FLAGS) -- $(ARGS)

clean:
	rm -rf vgcore* massif* target

dist:	default
	mkdir -p dmenu-$(VERSION)
	cp -r LICENSE README.md README makefile build.rs Cargo.toml src dmenu-$(VERSION)
	tar -cf dmenu-$(VERSION).tar dmenu-$(VERSION)
	gzip dmenu-$(VERSION).tar
	rm -rf dmenu-$(VERSION)
