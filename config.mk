# paths
PREFIX = /usr/local
MANPREFIX = $(PREFIX)/share/man

# Xinerama, set to false/empty if you don't want it
XINERAMA=true

# flags: -std=c99, -Wall, opt_level, and linking are taken care of automatically
CFLAGS = -pedantic

# compiler and linker for non-rust files
CC = clang
