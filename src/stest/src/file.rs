use clap::Parser;
use std::clone::Clone;
use std::fmt::Error as FmtError;
use std::fmt::{Display, Formatter};
use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::Metadata;
use std::io;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::Component;
use std::path::PathBuf;
use std::result::Result;
use std::str::FromStr;

/// Wraps a [PathBuf] for use in stest.
///
/// These files are assumed to be on a unix system.
///
/// Error-handling here employs two approaches: system errors (e.g., file IO) induce panic, user
/// erros (e.g., trying to process a file the user does not have access to) return a helpful error
/// message wrapped in a result, which composes up to the outer shell of the program.
#[derive(Clone, Debug, Parser, PartialEq)]
pub struct File {
    path_buf: PathBuf,
}

impl File {
    pub fn new(path_buf: PathBuf) -> File {
        File { path_buf }
    }

    pub fn from(string: String) -> File {
        let path_buf = PathBuf::from(string);
        File { path_buf }
    }

    pub fn read_directory(&self) -> Result<Vec<File>, std::io::Error> {
        fn dir_entry_to_file(dir_entry: DirEntry) -> File {
            File::new(dir_entry.path())
        }
        let iterator = read_dir(&self.path_buf)?;
        iterator
            .map(|result| result.map(dir_entry_to_file))
            .collect()
    }

    pub fn is_hidden(&self) -> bool {
        fn is_hidden(component: Component) -> bool {
            fn component_starts_with_dot(component: Component) -> bool {
                component
                    .as_os_str()
                    .to_string_lossy() // Ignore invalid unicode characters.
                    .starts_with('.')
            }
            component != Component::CurDir
                && component != Component::ParentDir
                && component_starts_with_dot(component)
        }
        let iterator = self.path_buf.as_path().components();
        // If a file's path is empty, it cannot be hidden.
        let option = iterator.last();
        option
            .map(|component| is_hidden(component))
            .unwrap_or(false)
    }

    pub fn is_block_special(&self) -> Result<bool, io::Error> {
        fn is_block_special(metadata: Metadata) -> bool {
            metadata.file_type().is_block_device()
        }
        self.metadata().map(is_block_special)
    }

    pub fn is_character_special(&self) -> Result<bool, io::Error> {
        fn is_character_special(metadata: Metadata) -> bool {
            metadata.file_type().is_char_device()
        }
        self.metadata().map(is_character_special)
    }

    pub fn is_directory(&self) -> bool {
        self.path_buf.is_dir()
    }

    pub fn exists(&self) -> Result<bool, io::Error> {
        self.path_buf.try_exists()
    }

    pub fn is_file(&self) -> bool {
        self.path_buf.is_file()
    }

    /// Check if the file has the set-group-ID bit set.
    /// See: https://stackoverflow.com/a/50045872/8732788
    /// See: https://en.wikipedia.org/wiki/Setuid
    pub fn has_set_group_id(&self) -> Result<bool, io::Error> {
        fn has_set_group_id(mode: u32) -> bool {
            mode & 0o2000 != 0
        }
        self.mode().map(has_set_group_id)
    }

    pub fn is_symbolic_link(&self) -> bool {
        self.path_buf.is_symlink()
    }

    pub fn is_newer_than(&self, file: &File) -> Result<bool, io::Error> {
        let modified_time = self.path_buf.metadata()?.modified()?;
        let oldest_modified_time = file.path_buf.metadata()?.modified()?;
        let bool = modified_time > oldest_modified_time;
        Ok(bool)
    }

    pub fn is_older_than(&self, file: &File) -> Result<bool, io::Error> {
        let modified_time = self.path_buf.metadata()?.modified()?;
        let newest_modified_time = file.path_buf.metadata()?.modified()?;
        let bool = modified_time < newest_modified_time;
        Ok(bool)
    }

    pub fn is_pipe(&self) -> Result<bool, io::Error> {
        fn is_pipe(metadata: Metadata) -> bool {
            metadata.file_type().is_fifo()
        }
        self.path_buf.metadata().map(is_pipe)
    }

    /// Check if a unix file has any readable bit set (user, group, or other).
    /// See: https://en.wikipedia.org/wiki/File-system_permissions#Numeric_notation
    /// See: https://en.wikipedia.org/wiki/Bitwise_operation#AND
    pub fn is_readable(&self) -> Result<bool, io::Error> {
        fn is_readable(mode: u32) -> bool {
            mode & 0o444 != 0
        }
        self.mode().map(is_readable)
    }

    pub fn has_size_greater_than_zero(&self) -> Result<bool, io::Error> {
        fn has_size_greater_than_zero(metadata: Metadata) -> bool {
            let len = metadata.len();
            len > 0
        }
        self.metadata().map(has_size_greater_than_zero)
    }

    /// Check if the file has the set-user-ID bit set.
    /// See: https://stackoverflow.com/a/50045872/8732788
    /// See: https://en.wikipedia.org/wiki/Setuid
    pub fn has_set_user_id(&self) -> Result<bool, io::Error> {
        fn has_set_user_id(mode: u32) -> bool {
            mode & 0o4000 != 0
        }
        self.mode().map(has_set_user_id)
    }

    /// Check if the file has any writable bit set (user, group, or other).
    /// See: https://en.wikipedia.org/wiki/File-system_permissions#Numeric_notation
    /// See: https://en.wikipedia.org/wiki/Bitwise_operation#AND
    pub fn is_writable(&self) -> Result<bool, io::Error> {
        fn is_writable(mode: u32) -> bool {
            mode & 0o222 != 0
        }
        self.mode().map(is_writable)
    }

    /// Check if the file has any executable bit set (user, group, or other).
    /// See: https://en.wikipedia.org/wiki/File-system_permissions#Numeric_notation
    /// See: https://en.wikipedia.org/wiki/Bitwise_operation#AND
    pub fn is_executable(&self) -> Result<bool, io::Error> {
        fn is_executable(mode: u32) -> bool {
            mode & 0o111 != 0
        }
        self.mode().map(is_executable)
    }

    pub fn clone_with_path_as_file_name(&self) -> Option<File> {
        self.path_buf
            .file_name()
            .map(|os_str| PathBuf::from(os_str))
            .map(|path_buf| File::new(path_buf))
    }

    fn metadata(&self) -> Result<Metadata, io::Error> {
        self.path_buf.metadata()
    }

    fn mode(&self) -> Result<u32, io::Error> {
        fn metadata_to_mode(metadata: Metadata) -> u32 {
            metadata.permissions().mode()
        }
        self.metadata().map(metadata_to_mode)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let display = self.path_buf.display();
        write!(f, "{display}")
    }
}

impl FromStr for File {
    type Err = std::convert::Infallible;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let result = PathBuf::from_str(str);
        result.map(&File::new)
    }
}
