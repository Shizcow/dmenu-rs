# dmenu-rs - dynamic menu
dmenu is an efficient dynamic menu for X.  
dmenu-rs is a 1:1 port of dmenu rewritten in Rust. It looks, feels, and
runs pixel-for-pixel exactly the same.  
It also has plugin support for easy modification.

## State of the project
The master branch is a stable, feature complete product. It it not unmaintained; it's finished.

There is a __small__ and ever-shrinking chance that this will be updated or overhauled in the distant future. It is much more likely that I will write a spiritual successor from scratch, but that won't happen until at least 2023.

## Why Rust?
### Inspiration
This project started with [`dmenu-calc`](https://github.com/sumnerevans/menu-calc).
Initially, I wanted much more function than what is provided by `bc`. However, I
found the bottleneck to be a lack of functionality and modability in `dmenu(1)`
itself. So, the choice I had was to either mod `dmenu(1)` or rewrite it. Because
dmenu source is horrendously annoying to read, I decided to rewrite it in a
language which lends itself to writing code that is easier to modify. There are
other languages for this, but I like Rust.
### Improvements
As mentioned earlier, `dmenu-rs` runs exactly the same as `dmenu`. However, there
are some significant performance enhancements under the hood. The most impactful
is memory usage: `dmenu-rs` uses 21.65% less memory<sup>[1]</sup>, while managing it much
more safely **without** any performance impacts. The other large improvement is
plugin support; read below.

## Plugins
dmenu-rs leverages rust crates `overrider` and `proc_use` to provide an easy to
write and powerful plugin system. The end-result are plugins which are dead-simple
to enable.  
For a list of available plugins and more info on
enabling plugins, run `make plugins`.  
For more info on developing plugins, read the [plugin guide](src/plugins/README.md).

## Requirements
- Xlib header files  
- Cargo / rustc  
- A working C compiler

## Installation
### Standalone
Edit config.mk to match your local setup (dmenu is installed into
the /usr/local namespace by default).

Afterwards enter the following command to build dmenu:  
```make```  
Then, to install (if necessary as root):  
```make install```
### Distros
dmenu-rs is available from the following sources:
- [Arch AUR - stable branch](https://aur.archlinux.org/packages/dmenu-rs/)  
- [Arch AUR - development branch](https://aur.archlinux.org/packages/dmenu-rs-git/)  

If you'd like for this to be available on another distro, raise an issue
or submit a pull request with a README change pointing to the released
package.

## Running dmenu
See the man page for details. For a quick test, run:  
```make test```

<br/><br/>
<sup>[1]</sup>: According to `valgrind(1)`
