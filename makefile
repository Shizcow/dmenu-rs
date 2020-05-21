# dmenu-rs - dynamic menu
# See LICENSE file for copyright and license details.

include config.mk

VERSION:=$(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)

ifeq ($(XINERAMA),true)
	XINERAMA_FLAGS = --features "Xinerama"
endif

ifeq ($(CC),)
	CC = cc
endif

export CC
export CFLAGS
export RUSTFLAGS

all:	options dmenu stest

options:
	@echo dmenu build options:
	@echo "CFLAGS     = $(CFLAGS)"
	@echo "CC         = $(CC)"
	@echo "RUSTFLAGS  = $(RUSTFLAGS)"

dmenu:
	cargo build --release $(XINERAMA_FLAGS)

test:
	seq 1 100 | cargo run --release $(XINERAMA_FLAGS) -- $(ARGS)

stest:
	mkdir -p target
	mkdir -p target/release
	$(CC) -c $(CFLAGS) -o target/release/stest.o src/stest/stest.c
	$(CC) -o target/release/stest target/release/stest.o
	rm -f target/release/stest.o

clean:
	rm -rf vgcore* massif* target

dist:	default
	mkdir -p dmenu-$(VERSION)
	cp -r LICENSE README.md README makefile build.rs Cargo.toml src dmenu-$(VERSION)
	tar -cf dmenu-$(VERSION).tar dmenu-$(VERSION)
	gzip dmenu-$(VERSION).tar
	rm -rf dmenu-$(VERSION)
