VERSION = 5.5.2

# paths
PREFIX = /usr/local
MANPREFIX = $(PREFIX)/share/man

# Xinerama, set to false/empty if you don't want it
XINERAMA=true

# compiler and linker for non-rust files, blank for system default (cc)
CC =

# additional flags to be passed to rustc
RUSTFLAGS =

# space seperated list of plugins to be compiled in
# run `make plugins` to see a list of available plugins
PLUGINS = 
