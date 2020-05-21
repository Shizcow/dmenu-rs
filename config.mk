# paths
PREFIX = /usr/local
MANPREFIX = $(PREFIX)/share/man

# Xinerama, set to false/empty if you don't want it
XINERAMA=true

# flags (not used for dmenu)
CFLAGS = -c -pedantic -std=c99 -Wall -Os -D_DEFAULT_SOURCE

# compiler and linker for non-rust files, blank for system default (cc)
CC = clang

# additional flags to be passed to rustc
RUSTFLAGS =

# paths for install
PREFIX = /usr/local
MANPREFIX = $(PREFIX)/share/man
