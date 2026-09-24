use std::fs::File;
use std::io::BufReader;

use jieba_rs::Jieba;

pub fn load_from_jieba_default_dict() -> Jieba {
    Jieba::new()
}

pub fn load_from_jieba_dict(path: &str) -> crate::Result<Jieba> {
    let mut reader = BufReader::new(File::open(path)?);
    Ok(Jieba::with_dict(&mut reader)?)
}
