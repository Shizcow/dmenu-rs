# headers

These files contain some basic includes for C++ libraries:
- fontconfig.h
- xinerama.h
- xlib.h

These files are parsed by bindgen, providing rust bindings
for some additional functions which the default x11 and
fontconfig crates don't provide or get incorrect.
