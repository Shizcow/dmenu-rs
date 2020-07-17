# Plugin development

0. [Introduction](#introduction)
1. [Directory Structure](#directory-structure)
2. [Building](#building)  
   a. [Testing Changes](#testing-changes)
3. [Files](#files)  
   a. [plugin.yml](#pluginyml)  
   b. [main.rs](#mainrs)  
   c. [deps.toml](#depstoml)
4. [Manpage Generation](#manpage-generation)
5. [What Functionality Can Be Changed](#what-functionality-can-be-changed)
6. [Quickstart](#quickstart)  
   a. [Cloning The Project](#cloning-the-project)  
   b. [Setting Up Files](#setting-up-files)  
   c. [Compiling The Plugin](#compiling-the-plugin)  
   d. [CompResult](#compresult)

## Introduction

Developing plugins for dmenu-rs is a relatively simple process. Because this project is still
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

### Testing Changes
As mentioned above, `config.mk` controls what plugins are loaded. Add your plugin name to the
`PLUGINS` field to have it compiled in.

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
build: "sh build.sh"

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
- build: This field is optional. If included, runs a command during configuration. This is
  useful for dependency checks or downloading external libraries that will be referenced
  during compile time. When checking dependencies, check `"$depcheck" != "false"` for
  runtime dependencies. An example of this check can be found in the `spellcheck` plugin.
- args: These are command line arguments. They have the same syntax as arguments for `clap`,
  and are more or less copy-pasted into a `cli.yml` file down the line.  
  Support for `visible_aliases` has been added in, so these work out of the box while `clap`
  does not yet support them.

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


## Manpage Generation
Manpages are automatically generated including info for plugin-based flags. The flag
long/short names are included if present. `help` is required, unless `long_help` is
provided. If both `help` and `long_help` are provided, `long_help` will be included
in manpage generation.

## What Functionality Can Be Changed
Because dmenu-rs is still young, not all functionality is changable yet.

For a living list of everything that can be overriden, see
[the plugin entry](../dmenu/plugin_entry.rs). Every method and funciton here is exposed
to `overrider`.

Some examples include:
- `gen_matches` for displaying dynamic menu content
- `format_input` for changing how the input renders on screen
- `ConfigDefault` methods, which set the default values of config variables  
More are on their way.

## Quickstart
Here's a short walkthrough on how to write a plugin, get the build system to recognize it,
and get changes working correctly.

This example will show how to make a `hello` plugin which replaces all menu items with
the phrase `Hello world!`.

### Cloning The Project
The first thing to do it get a working copy of this repo, and to make sure a clean build
is working. To do so, either fork and clone or run the following:
```
git clone https://github.com/Shizcow/dmenu-rs/
cd dmenu-rs
```
Now, switch to the `develop` branch. This branch has the latest features, so any build
conflicts will be more easily resolved here:
```
git checkout develop
```
Finally, make sure you have all the build tools installed:
```
make
```
This will check dependencies and attempt a build. If it doesn't succeed, you're likely
missing dependencies.

### Setting Up Files
The next thing to do is make plugin files. Switch to the plugin directory:
```
cd src/plugins
```
And make a plugin folder. The name of the folder is the name of the plugin. In this example,
we're making a `hello` plugin, so the command is as follows:
```
mkdir hello
cd hello
```
Now for the actual content. Create the following files:
```yaml
#plugin.yml

about: Replaces all menu items with the phrase "Hello world!"
entry: main.rs
```
```rust
#main.rs

use overrider::*;
use crate::drw::Drw;
use crate::item::Item;
use crate::result::*;

#[override_default]
impl Drw {
    pub fn gen_matches(&mut self) -> CompResult<Vec<Item>> {
	let mut ret = Vec::new();
	for _ in 0..self.get_items().len() {
	    ret.push(Item::new("Hello world!".to_owned(), false, self)?);
	}
	Ok(ret)
    }
}
```
As far as a basic plugin goes, that's all that's required.

### Compiling The Plugin
Now return back to the root of the project:
```
cd ../../
```
And open the `config.mk` file. Scroll to the bottom and you'll see the following:
```mk
PLUGINS = 
```
To compile with the new `hello` plugin, change that line to the following:
```mk
PLUGINS = hello
```
Finally, build the project:
```
make
```
Hopefully, everything will build correctly. Now to test changes:
```
make test
```
And voilà, menu output should be different. For more customization, see the rest of
the guide above.

### CompResult
`CompResult` is a type defined as follows:
```rust
pub type CompResult<T> = Result<T, Die>;
```
Where `Die` is defined as follows:
```rust
pub enum Die {
    Stdout(String),
    Stderr(String),
}
```
More info can be found in the [result.rs](../dmenu/result.rs) file.

The purpose of this type is stopping the program quickly. Be it in error,
or some reason that requires a quick-exit (such as auto-selection). Most
plugin_entry methods return `CompResult`, so quickly exiting from any point
is possible.
