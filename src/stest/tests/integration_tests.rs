//! These stest integration tests use executable bash files to set up their test fixtures (files
//! and directories on the local filesystem). The bash is written to try to be portable across
//! systems, but it may not be.
//!
//! Most tests share roughly the same structure. They set up both positive and negative test cases
//! (i.e., a file that will pass stest under a given configuration and a file that will not), then
//! generate a config struct and expected output, then run the stest app, and then assert that the
//! actual output equals the expected output. Comparisons are done using the File struct rather
//! than raw stdout byte streams, because it makes test failures easier to read.

use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::str;
use stest::config::Config;
use stest::file::File;
use stest::App;

#[test]
fn test_hidden_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_hidden = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-hidden-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_block_special_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_block_special = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-block-special-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_character_special_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_character_special = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-character-special-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_directory() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_directory = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-directory", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_exists() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_exists = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-file", "set-up-nonexisting-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_file = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-file", "set-up-directory");
    assert_eq!(actual, expected);
}

#[test]
fn test_set_group_id_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_has_set_group_id = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-file-with-set-group-id", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_symbolic_link() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_symbolic_link = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-symbolic-link", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_contents_of_directories() -> () {
    let (input, output) = {
        let directory = run_script("set-up-directory-with-contents");
        let contents = {
            directory
                .clone()
                .into_iter()
                .map(|file| file.read_directory().unwrap())
                .flatten()
                .map(|f| f.clone_with_path_as_file_name().unwrap())
                .collect()
        };
        (directory, contents)
    };

    let config = {
        let mut config = EMPTY.clone();
        config.test_contents_of_directories = true;
        config.files = input;
        config
    };
    let mut stdin: &[u8] = &[];
    let mut stdout: Vec<u8> = vec![];

    let app = App::new(config);
    let result = app.run(&mut stdin, &mut stdout);
    let actual = StestResult::new(result, stdout.to_files());

    let expected: StestResult = {
        let stdout = output;
        StestResult::new(Ok(true), stdout)
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_newer_than_oldest_file() -> () {
    let negative_case = run_script("set-up-file");
    let mut oldest_file = run_script("set-up-file");
    let positive_case = run_script("set-up-file");

    let (input, output) = positive_and_negative_to_input_and_output(positive_case, negative_case);

    let config = {
        let mut config = EMPTY.clone();
        config.oldest_file = oldest_file.pop();
        config.files = input;
        config
    };
    let mut stdin: &[u8] = &[];
    let mut stdout: Vec<u8> = vec![];

    let app = App::new(config);
    let result = app.run(&mut stdin, &mut stdout);
    let actual = StestResult::new(result, stdout.to_files());

    let expected: StestResult = {
        let stdout = output;
        StestResult::new(Ok(true), stdout)
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_older_than_newest_file() -> () {
    let positive_case = run_script("set-up-file");
    let mut newest_file = run_script("set-up-file");
    let negative_case = run_script("set-up-file");

    let (input, output) = positive_and_negative_to_input_and_output(positive_case, negative_case);

    let config = {
        let mut config = EMPTY.clone();
        config.newest_file = newest_file.pop();
        config.files = input;
        config
    };
    let mut stdin: &[u8] = &[];
    let mut stdout: Vec<u8> = vec![];

    let app = App::new(config);
    let result = app.run(&mut stdin, &mut stdout);
    let actual = StestResult::new(result, stdout.to_files());

    let expected: StestResult = {
        let stdout = output;
        StestResult::new(Ok(true), stdout)
    };

    assert_eq!(actual, expected);
}

#[test]
fn test_pipe_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_pipe = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-pipe-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_readable_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_readable = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-readable-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_nonempty_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_has_size_greater_than_zero = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-nonempty-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_set_user_id_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_has_set_user_id = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-file-with-set-user-id", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_writable_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_writable = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-writable-file", "set-up-file");
    assert_eq!(actual, expected);
}

#[test]
fn test_executable_file() -> () {
    let config = {
        let mut config = EMPTY.clone();
        config.requires_each_file_is_executable = true;
        config
    };
    let (actual, expected) = set_up_test(config, "set-up-executable-file", "set-up-file");
    assert_eq!(actual, expected);
}

static EMPTY: Config = Config {
    requires_each_file_is_hidden: false,
    requires_each_file_is_block_special: false,
    requires_each_file_is_character_special: false,
    requires_each_file_is_directory: false,
    requires_each_file_exists: false,
    requires_each_file_is_file: false,
    requires_each_file_has_set_group_id: false,
    requires_each_file_is_symbolic_link: false,
    test_contents_of_directories: false,
    oldest_file: None,
    newest_file: None,
    requires_each_file_is_pipe: false,
    quiet: false,
    requires_each_file_is_readable: false,
    requires_each_file_has_size_greater_than_zero: false,
    requires_each_file_has_set_user_id: false,
    has_inverted_tests: false,
    requires_each_file_is_writable: false,
    requires_each_file_is_executable: false,
    files: Vec::new(),
};

#[derive(Debug, PartialEq)]
struct StestResult {
    result: Result<bool, IOErrorWithPartialEq>,
    stdout: Vec<File>,
}

impl StestResult {
    fn new(result: Result<bool, io::Error>, stdout: Vec<File>) -> Self {
        let result = result.map_err(|io_error| IOErrorWithPartialEq(io_error));
        StestResult { result, stdout }
    }
}

/// io::Error does not define PartialEq for a good reason: what are the semantics of equality
/// between IO errors? Aren't they always (or nearly always) unique? Instead, we define a newtype
/// that has the semantics we want when comparing IO errors when integration testing here: they're
/// all equivalent.
/// See: https://users.rust-lang.org/t/help-understanding-io-error-and-lack-of-partialeq/13212/2
#[derive(Debug)]
struct IOErrorWithPartialEq(io::Error);

impl PartialEq for IOErrorWithPartialEq {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn set_up_test(
    config: Config,
    positive_case: &str,
    negative_case: &str,
) -> (StestResult, StestResult) {
    let (input, output): (Vec<File>, Vec<File>) =
        set_up_positive_and_negative_tests(positive_case, negative_case);

    let config = {
        let mut config = config.clone();
        config.files = input;
        config
    };
    let mut stdin: &[u8] = &[];
    let mut stdout: Vec<u8> = vec![];

    let app = App::new(config);
    let result = app.run(&mut stdin, &mut stdout);
    let actual = StestResult::new(result, stdout.to_files());

    let expected: StestResult = {
        let stdout = output;
        StestResult::new(Ok(true), stdout)
    };

    (actual, expected)
}

fn set_up_positive_and_negative_tests(
    positive_script_filename: &str,
    negative_script_filename: &str,
) -> (Vec<File>, Vec<File>) {
    let positive_cases = run_script(positive_script_filename);
    let negative_cases = run_script(negative_script_filename);
    positive_and_negative_to_input_and_output(positive_cases, negative_cases)
}

fn positive_and_negative_to_input_and_output(
    positive_cases: Vec<File>,
    negative_cases: Vec<File>,
) -> (Vec<File>, Vec<File>) {
    let input = {
        let mut vec = positive_cases.clone();
        vec.extend(negative_cases.clone());
        vec
    };
    let output = positive_cases;
    (input, output)
}

fn run_script(script: &str) -> Vec<File> {
    let path_buf: PathBuf = {
        let mut path_buf = {
            let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
            PathBuf::from(cargo_manifest_dir)
        };
        let string = format!("tests/{script}");
        path_buf.push(string);
        path_buf
    };
    let os_str = path_buf.as_path().as_os_str();
    Command::new(os_str)
        .output()
        .map(|output| output.stdout)
        .map(|vec| vec.to_files())
        .unwrap()
}

trait ToFiles {
    fn to_files(&self) -> Vec<File>;
}

impl ToFiles for Vec<u8> {
    fn to_files(&self) -> Vec<File> {
        let slice = self.as_slice();
        let str = str::from_utf8(slice).unwrap();
        // Note, we trim the final newline before splitting.
        let vec: Vec<&str> = str.trim_end().split('\n').collect();
        vec.into_iter()
            .map(|str| PathBuf::from(str))
            .map(|path_buf| File::new(path_buf))
            .collect()
    }
}

trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for Vec<File> {
    fn to_bytes(&self) -> Vec<u8> {
        self.into_iter()
            .map(|file| {
                let mut string = file.to_string();
                // Note, this appends a newline at the end of the list of files.
                string.push('\n');
                string
            })
            .map(|string| string.into_bytes())
            .flatten()
            .collect()
    }
}
