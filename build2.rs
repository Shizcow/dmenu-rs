
    // bindings depend on files in the headers directory, so make sure they are tracked for rebuild on edit
    for e in WalkDir::new("headers").into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() {
	    let name = e.path().to_str().unwrap();
	    if name.as_bytes()[name.len()-1] != '~' as u8 { // ignore editor files
		println!("cargo:rerun-if-changed={}", e.path().display());
	    }
	}
    }
