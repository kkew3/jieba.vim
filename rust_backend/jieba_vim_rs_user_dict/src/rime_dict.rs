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
use std::io::{self, BufRead, BufReader, Cursor, Read};
use std::path::{Path, PathBuf};

use jieba_rs::Jieba;
use serde::Deserialize;

/// Split Rime *.dict.yaml into the yaml header and non-yaml payload, ignoring
/// surrounding comments.
fn split_rime_dict(content: &str) -> Option<(&str, &str)> {
    let mut offset = 0;
    let mut header_start = None;
    let mut header_stop = None;
    let mut payload_start = None;
    for line in content.split_inclusive('\n') {
        let line_trim = line.trim_ascii();
        if line_trim.is_empty() || line_trim.starts_with('#') {
            offset += line.len();
            continue;
        }
        if line_trim == "---" {
            offset += line.len();
            header_start = Some(offset);
            continue;
        }
        if line_trim == "..." {
            header_stop = Some(offset);
            offset += line.len();
            payload_start = Some(offset);
            continue;
        }
        offset += line.len();
    }
    let header_start = header_start?;
    let header_stop = header_stop?;
    let payload_start = payload_start?;
    Some((
        &content[header_start..header_stop],
        &content[payload_start..],
    ))
}

fn split_rime_dict_reader(
    mut reader: impl BufRead,
) -> crate::Result<(String, impl BufRead)> {
    let mut buf = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let size = reader.read_line(&mut line)?;
        if size == 0 {
            break; // eof
        }
        buf.push_str(&line);
        if let Some((_, payload)) = split_rime_dict(&buf) {
            if !payload.is_empty() {
                break;
            }
        }
    }
    // Till here, we have either a non-empty payload, or the entire file fit
    // into `buf`. In either case, `split_rime_dict` will return complete yaml
    // header in the first returned slice.
    let (header, payload_prefix) = split_rime_dict(&buf).ok_or_else(|| {
        crate::Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid dict.yaml file: yaml header not found",
        ))
    })?;
    Ok((
        header.to_owned(),
        Cursor::new(payload_prefix.to_owned()).chain(reader),
    ))
}

fn default_columns() -> Vec<String> {
    vec!["text".into(), "code".into(), "weight".into()]
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct YamlHeader {
    #[serde(default = "default_columns")]
    columns: Vec<String>,
    #[serde(default)]
    import_tables: Vec<String>,
}

fn load_from_rime_dict_helper(
    path: impl AsRef<Path>,
    jieba: &mut Jieba,
) -> crate::Result<YamlHeader> {
    let reader = BufReader::new(File::open(path)?);
    let (header, reader) = split_rime_dict_reader(reader)?;
    let header: YamlHeader = serde_yaml_ng::from_str(&header)?;
    for line in reader.lines() {
        let line = line?;
        let mut text = None;
        let mut weight: Option<usize> = None;
        for (token, field_name) in line.split('\t').zip(&header.columns) {
            if field_name == "text" {
                text = text.or(Some(token));
            } else if field_name == "weight" {
                weight = weight.or(token.parse().ok());
            }
        }
        if let Some(text) = text {
            if text.is_empty() || text.chars().all(|c| c.is_ascii_whitespace())
            {
                continue;
            }

            // `+ 1`: jieba dict expects the minimum frequency to be 2 rather
            // than 1, whereas the minimum weight in Rime dict is 1.
            jieba.add_word(text, weight.map(|f| f + 1), None);
        }
    }
    Ok(header)
}

pub fn load_from_rime_dict(path: &str) -> crate::Result<Jieba> {
    let mut jieba = Jieba::empty();
    let header = load_from_rime_dict_helper(path, &mut jieba)?;
    let path: PathBuf = path.into();
    if let Some(dir) = path.parent() {
        for name in header.import_tables {
            let imported_path = dir.join(format!("{name}.dict.yaml"));
            load_from_rime_dict_helper(imported_path, &mut jieba)?;
        }
    }
    Ok(jieba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_rime_dict_no_payload1() {
        let content =
            include_str!("test_rime_dict_samples/rime_ice.dict.yaml_");
        let (header, payload) = split_rime_dict(content).unwrap();
        assert_eq!(
            header,
            r#"name: rime_ice
version: "2026-01-26"
import_tables:
  - cn_dicts/8105     # 字表
  # - cn_dicts/41448  # 大字表（按需启用）（启用时和 8105 同时启用并放在 8105 下面）
  - cn_dicts/base     # 基础词库
  - cn_dicts/ext      # 扩展词库
  - cn_dicts/tencent  # 腾讯词向量（大词库，部署时间较长）
  - cn_dicts/others   # 一些杂项

  # 建议把扩展词库放到下面，有重复词条时，最上面的权重生效
  # - mydict1           # 挂载配置目录下的 mydict1.dict.yaml 词库文件
  # - cn_dicts/mydict2  # 挂载 cn_dicts 目录里的 mydict2.dict.yaml 词库文件
"#
        );
        assert_eq!(payload, "");
    }

    #[test]
    fn test_split_rime_dict_no_payload2() {
        let content =
            include_str!("test_rime_dict_samples/cangjie6.extended.dict.yaml_");
        let (header, payload) = split_rime_dict(content).unwrap();
        assert_eq!(
            header,
            r#"sort: by_weight
use_preset_vocabulary: false
import_tables:
  - cangjie6 #單字碼表由cangjie6.dict.yaml導入
columns: #此字典爲純詞典，無單字編碼，僅有字和詞頻
  - text #字／詞
  - weight #字／詞頻
encoder:
  exclude_patterns:
    - '^z.*$'
  rules:
    - length_equal: 2 #對於二字詞
      formula: "AaAzBaBbBz" #取第一字首尾碼、第二字首次尾碼
    - length_equal: 3 #對於三字詞
      formula: "AaAzBaYzZz" #取第一字首尾碼、第二字首尾碼、第三字尾碼
    - length_in_range: [4, 5] #對於四至五字詞
      formula: "AaBzCaYzZz" #取第一字首碼，第二字尾碼、第三字首碼、倒數第二字尾碼、最後一字尾碼
  tail_anchor: "'"
"#
        );
        assert_eq!(payload, "");
    }

    #[test]
    fn test_split_rime_dict1() {
        let content = include_str!("test_rime_dict_samples/base.dict.yaml_");
        let (header, payload) = split_rime_dict(content).unwrap();
        assert_eq!(
            header,
            r#"name: base
version: "2026-08-16"
sort: by_weight
"#
        );
        assert_eq!(
            payload,
            r#"# +_+
啊啊	a a	27365
啊啊啊	a a a	9820
啊啊啊啊	a a a a	9820
啊嗷	a ao	111
"#
        );
    }

    #[test]
    fn test_split_rime_dict2() {
        let content = include_str!("test_rime_dict_samples/8105.dict.yaml_");
        let (header, payload) = split_rime_dict(content).unwrap();
        assert_eq!(
            header,
            r#"name: 8105
version: "2026-08-16"
sort: by_weight
"#
        );
        assert_eq!(
            payload,
            r#"### 按需启用
# 「这、那」的口语读音
这	zhei	17648808
# 那	nei	9929703


### 扩展，不在《通用规范汉字表》的字:
揞	an	1
鿫	ao	1
"#
        );
    }

    #[test]
    fn test_deserialize_yamlheader1() {
        let content =
            include_str!("test_rime_dict_samples/rime_ice.dict.yaml_");
        let (header, _) = split_rime_dict(content).unwrap();
        let header: YamlHeader = serde_yaml_ng::from_str(header).unwrap();
        assert_eq!(
            header,
            YamlHeader {
                columns: vec!["text".into(), "code".into(), "weight".into()],
                import_tables: vec![
                    "cn_dicts/8105".into(),
                    "cn_dicts/base".into(),
                    "cn_dicts/ext".into(),
                    "cn_dicts/tencent".into(),
                    "cn_dicts/others".into()
                ],
            }
        )
    }

    #[test]
    fn test_deserialize_yamlheader2() {
        let content = include_str!("test_rime_dict_samples/base.dict.yaml_");
        let (header, _) = split_rime_dict(content).unwrap();
        let header: YamlHeader = serde_yaml_ng::from_str(header).unwrap();
        assert_eq!(
            header,
            YamlHeader {
                columns: vec!["text".into(), "code".into(), "weight".into()],
                import_tables: vec![],
            }
        )
    }

    #[test]
    fn test_deserialize_yamlheader3() {
        let content =
            include_str!("test_rime_dict_samples/cangjie6.extended.dict.yaml_");
        let (header, _) = split_rime_dict(content).unwrap();
        let header: YamlHeader = serde_yaml_ng::from_str(header).unwrap();
        assert_eq!(
            header,
            YamlHeader {
                columns: vec!["text".into(), "weight".into()],
                import_tables: vec!["cangjie6".into()],
            }
        )
    }

    #[test]
    fn test_split_rime_dict_reader_no_payload1() {
        let reader = Cursor::new(include_bytes!(
            "test_rime_dict_samples/rime_ice.dict.yaml_"
        ));
        let (header, mut reader) = split_rime_dict_reader(reader).unwrap();
        let mut payload = String::new();
        reader.read_to_string(&mut payload).unwrap();
        assert_eq!(
            header,
            r#"name: rime_ice
version: "2026-01-26"
import_tables:
  - cn_dicts/8105     # 字表
  # - cn_dicts/41448  # 大字表（按需启用）（启用时和 8105 同时启用并放在 8105 下面）
  - cn_dicts/base     # 基础词库
  - cn_dicts/ext      # 扩展词库
  - cn_dicts/tencent  # 腾讯词向量（大词库，部署时间较长）
  - cn_dicts/others   # 一些杂项

  # 建议把扩展词库放到下面，有重复词条时，最上面的权重生效
  # - mydict1           # 挂载配置目录下的 mydict1.dict.yaml 词库文件
  # - cn_dicts/mydict2  # 挂载 cn_dicts 目录里的 mydict2.dict.yaml 词库文件
"#
        );
        assert_eq!(payload, "");
    }

    #[test]
    fn test_split_rime_dict_reader_no_payload2() {
        let reader = Cursor::new(include_bytes!(
            "test_rime_dict_samples/cangjie6.extended.dict.yaml_"
        ));
        let (header, mut reader) = split_rime_dict_reader(reader).unwrap();
        let mut payload = String::new();
        reader.read_to_string(&mut payload).unwrap();
        assert_eq!(
            header,
            r#"sort: by_weight
use_preset_vocabulary: false
import_tables:
  - cangjie6 #單字碼表由cangjie6.dict.yaml導入
columns: #此字典爲純詞典，無單字編碼，僅有字和詞頻
  - text #字／詞
  - weight #字／詞頻
encoder:
  exclude_patterns:
    - '^z.*$'
  rules:
    - length_equal: 2 #對於二字詞
      formula: "AaAzBaBbBz" #取第一字首尾碼、第二字首次尾碼
    - length_equal: 3 #對於三字詞
      formula: "AaAzBaYzZz" #取第一字首尾碼、第二字首尾碼、第三字尾碼
    - length_in_range: [4, 5] #對於四至五字詞
      formula: "AaBzCaYzZz" #取第一字首碼，第二字尾碼、第三字首碼、倒數第二字尾碼、最後一字尾碼
  tail_anchor: "'"
"#
        );
        assert_eq!(payload, "");
    }

    #[test]
    fn test_split_rime_dict_reader1() {
        let reader = Cursor::new(include_bytes!(
            "test_rime_dict_samples/base.dict.yaml_"
        ));
        let (header, mut reader) = split_rime_dict_reader(reader).unwrap();
        let mut payload = String::new();
        reader.read_to_string(&mut payload).unwrap();
        assert_eq!(
            header,
            r#"name: base
version: "2026-08-16"
sort: by_weight
"#
        );
        assert_eq!(
            payload,
            r#"# +_+
啊啊	a a	27365
啊啊啊	a a a	9820
啊啊啊啊	a a a a	9820
啊嗷	a ao	111
"#
        );
    }

    #[test]
    fn test_split_rime_dict_reader2() {
        let reader = Cursor::new(include_bytes!(
            "test_rime_dict_samples/8105.dict.yaml_"
        ));
        let (header, mut reader) = split_rime_dict_reader(reader).unwrap();
        let mut payload = String::new();
        reader.read_to_string(&mut payload).unwrap();
        assert_eq!(
            header,
            r#"name: 8105
version: "2026-08-16"
sort: by_weight
"#
        );
        assert_eq!(
            payload,
            r#"### 按需启用
# 「这、那」的口语读音
这	zhei	17648808
# 那	nei	9929703


### 扩展，不在《通用规范汉字表》的字:
揞	an	1
鿫	ao	1
"#
        );
    }
}
