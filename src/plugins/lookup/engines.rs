// Edit the following to add/remove/change search engines
// The keys are the parameters for the --engine arguement
// The values are the URLs. "%s" will be replaced with the search string.
pub static ENGINES: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "ddg" => "https://duckduckgo.com/%s",
    "crates" => "https://crates.io/crates/%s",
    "docs" => "https://docs.rs/%s",
    "rust" => "https://doc.rust-lang.org/std/?search=%s",
    "github" => "https://github.com/search?q=%s",
    "archwiki" => "https://wiki.archlinux.org/index.php?search=%s",
    "dictionary" => "https://www.merriam-webster.com/dictionary/%s",
    "thesaurus" => "https://www.merriam-webster.com/thesaurus/%s",
};
