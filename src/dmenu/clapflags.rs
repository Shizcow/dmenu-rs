use clap::{ArgMatches, App};
use yaml_rust::yaml::Yaml;
lazy_static::lazy_static! {
    static ref YAML: Yaml = {
        clap::YamlLoader::load_from_str(include_str!(concat!(env!("OUT_DIR"), "/cli.yml")))
            .expect("failed to load YAML file") 
            .pop()
            .unwrap()
        };
    pub static ref CLAP_FLAGS: ArgMatches<'static> = App::from_yaml(&YAML).get_matches();     
}
