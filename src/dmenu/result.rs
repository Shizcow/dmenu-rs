pub type CompResult<T> = Result<T, Die>;

pub enum Die {
    Stdout(String),
    Stderr(String),
}

impl<T> From<Die> for CompResult<T> {
    fn from(error: Die) -> Self {
	Err(error)
    }
}

impl Die {
    pub fn stdout<T>(msg: String) -> CompResult<T> {
	Self::Stdout(msg).into()
    }
    pub fn stderr<T>(msg: String) -> CompResult<T> {
	Self::Stderr(msg).into()
    }
}
