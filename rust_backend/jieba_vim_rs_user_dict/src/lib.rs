pub mod jieba_dict;
pub mod rime_dict;

use std::io;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Jieba(jieba_rs::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<jieba_rs::Error> for Error {
    fn from(err: jieba_rs::Error) -> Self {
        Self::Jieba(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

impl From<serde_yaml_ng::Error> for Error {
    fn from(err: serde_yaml_ng::Error) -> Self {
        Self::Io(io::Error::new(io::ErrorKind::InvalidData, err))
    }
}
