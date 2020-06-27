# Plugin development

Developing plugins for dmenu-rs is an overall simple process. Because this project is still
young, only a small part of the internal API is exposed. Raise an issue if you'd like
something specific exposed, or if instructions here are unclear.

If you write a decent plugin, feel welcome to submit a pull request to have it included upstream.

## Directory structure
The basic structure of a plugin is as follows, starting from project root:
```
dmenu-rs
└─src
  └─plugins
    └─PLUGIN_NAME
      └─plugin.yml
      └─main.rs
      └─deps.toml
```
To start developing a new plugin, simply add a new folder to `src/plugins` with the name the
same as the plugin name. The individual files are described in the **Files** section below.

## Building
The build process for plugins is dead-simple. During the process of running `make`, a few
things will be automatically configured:
- `config.mk` is read to determine which plugins should be compiled in
- Command line arguments are added to `clap`, so they will work out of the box
- Command line arguments are added to man pages
- Additional Rust crate dependencies are added to `Cargo.toml` for final compilation
- Plugin files are watched by `overrider` and `proc_use` so `rustc` compiles them in  
This all happens automatically, so no build script configuration is required.

## Files
Above described is the directory structure of a plugin. A more thorough explanation on each
is as follows:
### plugin.yml
dmenu-rs configures plugins using YAML. Why YAML and not TOML? Because `clap` uses YAML, and
work has not yet been done to change that yet.  
The structure of a `plugin.yml` file is as follows:
```yaml
about: A short description of this plugin
entry: main.rs
cargo_dependencies: deps.toml

args:
	$CLAP_ARGS
```
For each field:  
- about: This text is shown when `make plugins` is ran
- entry: This file is where all top-level overrides should occur. Utility functions can
  be defined elsewhere, but anything with `#[override_default]` must be here. The name
  is left up to the developer.
- cargo_dependencies: This field is optional. If additional crate dependencies are required,
  this field points to this file.
- args: These are command line arguments. They have the same syntax as arguments for `clap`,
  and are more or less copy-pasted into a `cli.yml` file down the line.

### main.rs
This file's actual name is set by the `entry` field in `plugin.yml`.

This is where all top-level overrides reside. Utility functions may be defined elsewhere,
and `mod`'d into compilation. Or, utility functions may be defined directly in this file, if
they are required at all.

Plugins function by overriding certain features. These can be overridden for all cases, or
only when specific flags are called. For more information on syntax, see the 
[`overrider`](https://docs.rs/overrider/0.6.1/overrider/) crate.

For a list of functions which can be overridden, see the `src/dmenu/plugin_entry.rs` file.  
Viewing examples of pre-existing plugins will be highly helpful.

### deps.toml
This file is optional, and it's actual name is set by the `cargo_dependencies` field in
`plugin.yml`.

This file contains additional crate dependencies required by a plugin. The text in this
file is literally copy-pasted directly into the `[dependencies]` section of `Cargo.toml`,
so all syntax is allowed.
