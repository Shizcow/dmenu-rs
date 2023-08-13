pub mod config;
pub mod file;
pub mod semigroup;

use crate::config::Config;
use crate::file::File;
use crate::semigroup::Semigroup;
use std::io;
use std::io::{BufRead, Write};

pub struct App {
    pub config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(self, stdin: &mut dyn BufRead, stdout: &mut dyn Write) -> Result<bool, io::Error> {
        let files = self.files(stdin)?;
        let passing_files = self.passing_files(files, stdout)?;
        if passing_files.is_empty() {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn files(&self, stdin: &mut dyn BufRead) -> Result<Vec<File>, io::Error> {
        let files = self.config.files.clone();

        let files = if files.is_empty() {
            stdin
                .lines()
                .map(|result| result.map(File::from))
                .collect::<Result<_, _>>()?
        } else {
            files
        };

        let files = if self.config.test_contents_of_directories {
            self.expand_directories(files)?
        } else {
            files
        };

        Ok(files)
    }

    fn expand_directories(&self, files: Vec<File>) -> Result<Vec<File>, io::Error> {
        files
            .into_iter()
            .map(|file| self.expand_directory(file))
            .reduce(|x, y| x.combine(y))
            .unwrap_or(Ok(vec![]))
    }

    fn expand_directory(&self, file: File) -> Result<Vec<File>, io::Error> {
        if file.is_directory() {
            file.read_directory()
        } else {
            Ok(vec![file])
        }
    }

    pub fn passing_files(
        &self,
        files: Vec<File>,
        stdout: &mut dyn Write,
    ) -> Result<Vec<File>, io::Error> {
        fn by_test(result: &Result<TestedFile, io::Error>) -> bool {
            *result
                .as_ref()
                .map(|tested_file| {
                    let TestedFile { file: _, passes } = tested_file;
                    passes
                })
                .unwrap_or(&true) // Pass errors through the filter for later.
        }

        let passing_files: Vec<File> = files
            .into_iter()
            .map(|file| self.test_file(file))
            .filter(by_test)
            .map(|result| result.map(|tested_file| tested_file.file))
            .map(|result| result.and_then(|file| self.write(stdout, file)))
            .collect::<Result<_, _>>()?;

        stdout.flush()?;

        Ok(passing_files)
    }

    fn test_file(&self, file: File) -> Result<TestedFile, io::Error> {
        let passes = self.test(&file)?;
        let tested_file = TestedFile { file, passes };
        Ok(tested_file)
    }

    /// Test that a file passes all configured tests.
    ///
    /// Each test is optional. If a test is disabled, the result is true by default. Only if it's
    /// enabled do we test the given file. This is expressed with the following boolean logic
    ///
    ///   !config.is_test_enabled || test
    ///
    /// We combine the result of each individual test with logical AND and return the result,
    /// inverting it if necessary.
    fn test(&self, file: &File) -> Result<bool, io::Error> {
        let passes_all_tests = (!self.config.requires_each_file_is_hidden || file.is_hidden())
            && (!self.config.requires_each_file_is_block_special || file.is_block_special()?)
            && (!self.config.requires_each_file_is_character_special
                || file.is_character_special()?)
            && (!self.config.requires_each_file_is_directory || file.is_directory())
            && (!self.config.requires_each_file_exists || file.exists()?)
            && (!self.config.requires_each_file_is_file || file.is_file())
            && (!self.config.requires_each_file_has_set_group_id || file.has_set_group_id()?)
            && (!self.config.requires_each_file_is_symbolic_link || file.is_symbolic_link())
            && (self.optionally_test_if_newer_than_oldest_file(file)?)
            && (self.optionally_test_if_older_than_newest_file(file)?)
            && (!self.config.requires_each_file_is_pipe || file.is_pipe()?)
            && (!self.config.requires_each_file_is_readable || file.is_readable()?)
            && (!self.config.requires_each_file_has_size_greater_than_zero
                || file.has_size_greater_than_zero()?)
            && (!self.config.requires_each_file_has_set_user_id || file.has_set_user_id()?)
            && (!self.config.requires_each_file_is_writable || file.is_writable()?)
            && (!self.config.requires_each_file_is_executable || file.is_executable()?);

        if self.config.has_inverted_tests {
            Ok(!passes_all_tests)
        } else {
            Ok(passes_all_tests)
        }
    }

    fn optionally_test_if_newer_than_oldest_file(&self, file: &File) -> Result<bool, io::Error> {
        let default = Ok(true);
        self.config
            .oldest_file
            .as_ref()
            .map(|oldest_file| file.is_newer_than(oldest_file))
            .unwrap_or(default)
    }

    fn optionally_test_if_older_than_newest_file(&self, file: &File) -> Result<bool, io::Error> {
        let default = Ok(true);
        self.config
            .newest_file
            .as_ref()
            .map(|newest_file| file.is_older_than(newest_file))
            .unwrap_or(default)
    }

    fn write(&self, stdout: &mut dyn Write, file: File) -> Result<File, io::Error> {
        if self.config.quiet {
            ()
        } else {
            let mut string = if self.config.test_contents_of_directories {
                file.clone_with_path_as_file_name()
                    .map(|f| f.to_string())
                    .expect("contents of the directory won't include '..'")
            } else {
                file.to_string()
            };
            string.push('\n');
            let bytes = string.as_bytes();
            stdout.write_all(bytes)?
        }
        Ok(file)
    }
}

struct TestedFile {
    file: File,
    passes: bool,
}
