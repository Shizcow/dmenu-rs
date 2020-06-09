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
export PLUGINS

all:	options dmenu stest

options:
	@echo dmenu build options:
	@echo "CFLAGS     = $(CFLAGS)"
	@echo "CC         = $(CC)"
	@echo "RUSTFLAGS  = $(RUSTFLAGS)"
	@echo "PLUGINS    = $(PLUGINS)"

dmenu:
	cargo build --release $(XINERAMA_FLAGS)

test:
	seq 1 100 | cargo run --release $(XINERAMA_FLAGS) -- $(ARGS)

stest:
	mkdir -p target
	mkdir -p target/release
	$(CC) $(CFLAGS) -o target/release/stest.o src/stest/stest.c
	$(CC) -o target/release/stest target/release/stest.o
	rm -f target/release/stest.o
	cp src/man/src/stest.1 target/release

clean:
	rm -rf vgcore* massif* target

dist:	
	mkdir -p dmenu-$(VERSION)
	cp -r LICENSE README.md README makefile build.rs Cargo.toml src dmenu-$(VERSION)
	tar -cf dmenu-$(VERSION).tar dmenu-$(VERSION)
	gzip dmenu-$(VERSION).tar
	rm -rf dmenu-$(VERSION)

# compile first, install with sudo if needed
install:
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	cp -f target/release/dmenu src/sh/dmenu_path src/sh/dmenu_run target/release/stest $(DESTDIR)$(PREFIX)/bin/
	chmod 755 $(DESTDIR)$(PREFIX)/bin/dmenu
	chmod 755 $(DESTDIR)$(PREFIX)/bin/dmenu_path
	chmod 755 $(DESTDIR)$(PREFIX)/bin/dmenu_run
	chmod 755 $(DESTDIR)$(PREFIX)/bin/stest
	mkdir -p $(DESTDIR)$(MANPREFIX)/man1
	cp target/release/dmenu.1 $(DESTDIR)$(MANPREFIX)/man1/dmenu.1
	sed "s/VERSION/$(VERSION)/g" < target/release/stest.1 > $(DESTDIR)$(MANPREFIX)/man1/stest.1
	chmod 644 $(DESTDIR)$(MANPREFIX)/man1/dmenu.1
	chmod 644 $(DESTDIR)$(MANPREFIX)/man1/stest.1

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/dmenu\
		$(DESTDIR)$(PREFIX)/bin/dmenu_path\
		$(DESTDIR)$(PREFIX)/bin/dmenu_run\
		$(DESTDIR)$(PREFIX)/bin/stest\
		$(DESTDIR)$(MANPREFIX)/man1/dmenu.1\
		$(DESTDIR)$(MANPREFIX)/man1/stest.1
