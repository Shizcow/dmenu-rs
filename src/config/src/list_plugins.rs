use std::path::PathBuf;
use glob::glob;
use prettytable::{format::Alignment, Attr, Cell, Row, Table, color};

mod util;
use util::*;

// list all the available plugins, along with their descriptions, args, etc
fn main() {
    let selected_plugins = get_selected_plugin_list();
    
    let mut table_head = Table::new();
    table_head.add_row(Row::new(vec![
	Cell::new("Available Plugins")
	    .with_style(Attr::Bold)
	    .with_style(Attr::ForegroundColor(color::BLUE))
    ]));
    table_head.printstd();

    
    let mut table = Table::new();
    
    table.set_titles(Row::new(vec![
	Cell::new_align("Sel", Alignment::CENTER)
	    .with_style(Attr::Bold),
	Cell::new_align("Name", Alignment::CENTER)
	    .with_style(Attr::Bold),
	Cell::new_align("Description", Alignment::CENTER)
	    .with_style(Attr::Bold),
    ]));

    let mut plugin_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    plugin_path.pop();
    plugin_path = plugin_path.join("plugins").join("*").join("plugin.yml");
    
    for entry in glob(&plugin_path.display().to_string())
	.expect("Failed to read glob pattern") {
	match entry {
            Err(e) => println!("{:?}", e),
            Ok(path) => {
		let plugin_name = path
		    .parent()   .unwrap()
		    .file_name().unwrap()
		    .to_str()   .unwrap()
		    .to_string();
		let mut plugin_yaml = get_yaml(&path.display().to_string(), None);
		let about = get_yaml_top_level(&mut plugin_yaml, "about").unwrap();

		table.add_row(Row::new(vec![
		    Cell::new_align(if selected_plugins.contains(&plugin_name) {
			"X"
		    } else {
			" "
		    }, Alignment::CENTER),
		    Cell::new(&plugin_name)
			.with_style(Attr::Bold)
			.with_style(Attr::ForegroundColor(color::GREEN)),
		    Cell::new(about)
			.with_style(Attr::Italic(true))
		]));
	    },
	}
    }
    
    table.printstd();

    let mut table_footer = Table::new();
    table_footer.add_row(Row::new(vec![
	Cell::new("To enable plugins, read the PATCH section in config.mk")
	    .with_style(Attr::Bold)
	    .with_style(Attr::ForegroundColor(color::BLUE))
    ]));
    table_footer.printstd();
}
