use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use itertools::Itertools;

pub struct Manpage {
    name: String,
    version: String,
    section: u32,
    desc_short: String,
    descriptions: Vec<(String, String)>
}

impl Manpage {
    pub fn new(name: &str, version: &str, section: u32) -> Self {
	Self {
	    name: name.to_string(),
	    version: version.to_string(),
	    section,
	    desc_short: String::new(),
	    descriptions: Vec::new(),
	}
    }

    pub fn desc_short(&mut self, desc_short: &str) -> &mut Self {
	self.desc_short = desc_short.to_string();
	self
    }

    pub fn description(&mut self, name: &str, desc: &str) -> &mut Self {
	self.descriptions.push((name.to_string(), desc.to_string()));
	self
    }

    pub fn write_to_file(&self, path: PathBuf) {
	let heading = format!(".TH {} {} {}\\-{}",
			      self.name.to_uppercase(),
			      self.section,
			      self.name,
			      self.version);
	let name = format!(".SH NAME\n{} \\- {}", self.name, self.desc_short);

	let description = format!(".SH DESCRIPTION\n{}",
				  self.descriptions.iter().map(|(name, description)| {
				      format!(".B {}\n{}", name, description)
				  }).join("\n.P\n"));
	
	let manpage = vec![heading, name, description].join("\n");
	match File::create(&path) {
	    Ok(mut file) => {
		if let Err(err) = file.write_all(manpage.as_bytes()) {
		    panic!("Could not write to file '{}': {}", path.to_string_lossy(), err);
		}
	    },
	    Err(err) => panic!("Could not open file '{}' for writing: {}",
			       path.to_string_lossy(), err),
	}
    }
}
