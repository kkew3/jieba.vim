// Copyright 2026 Kaiwen Wu. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

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
