/// This is the CompResult type
/// It is used across the crate to propogate errors more easily
pub type CompResult<T> = Result<T, Die>;

/// Its error is a Die
/// When dieing, the the following options are given:
/// - Stdout: print to stdout, exit with code 0
/// - Stderr: print to stderr, exit with code 1
/// If an empty string is returned, nothing is printed
/// but return codes are obeyed
pub enum Die {
    Stdout(String),
    Stderr(String),
}

/// The following are convienence methods for creating a Die
/// For example, instead of the following:
/// `return Err(Die::Stderr("fatal flaw".to_owned))`
/// Use this instead:
/// `return Die::stderr("fatal flaw".to_owned)`
impl Die {
    pub fn stdout<T>(msg: String) -> CompResult<T> {
	Self::Stdout(msg).into()
    }
    pub fn stderr<T>(msg: String) -> CompResult<T> {
	Self::Stderr(msg).into()
    }
}

impl<T> From<Die> for CompResult<T> {
    fn from(error: Die) -> Self {
	Err(error)
    }
}
