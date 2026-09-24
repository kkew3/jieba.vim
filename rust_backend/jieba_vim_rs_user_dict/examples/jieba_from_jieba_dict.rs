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

mod eval;

use std::fs::File;
use std::io::{self, BufRead, BufReader};

use clap::Parser;
use jieba_rs::Jieba;
use jieba_vim_rs_user_dict::jieba_dict::{
    load_from_jieba_default_dict, load_from_jieba_dict,
};

/// Run Chinese word segmentation and optionally evaluate against a groundtruth
/// corpus.
#[derive(Parser)]
struct Cli {
    /// The input unsegmented sentences path.
    text_file: String,

    /// The jieba dict path. Don't specify this arg to load the default dict.
    jieba_dict_path: Option<String>,

    /// The groundtruth corpus path. Don't specify this option to skip
    /// evaluating boundary scores and accuracies.
    #[arg(short = 'c')]
    groundtruth_corpus: Option<String>,
}

fn main() -> Result<(), jieba_vim_rs_user_dict::Error> {
    let cli = Cli::parse();
    let jieba = match cli.jieba_dict_path {
        None => load_from_jieba_default_dict(),
        Some(path) => load_from_jieba_dict(&path)?,
    };
    let text_to_cut = BufReader::new(File::open(cli.text_file)?);
    match cli.groundtruth_corpus {
        None => cut_text(&jieba, text_to_cut)?,
        Some(path) => {
            let groundtruth_corpus = BufReader::new(File::open(path)?);
            cut_eval(&jieba, text_to_cut, groundtruth_corpus)?;
        }
    }
    Ok(())
}

fn cut_text(jieba: &Jieba, text: impl BufRead) -> io::Result<()> {
    for line in text.lines() {
        let line = line?;
        let mut tokens = jieba.cut(&line, true).into_iter();
        if let Some(token1) = tokens.next() {
            print!("{}", token1.word);
            for token in tokens {
                print!(" {}", token.word);
            }
        }
        println!();
    }
    Ok(())
}

fn cut_eval(
    jieba: &Jieba,
    text: impl BufRead,
    groundtruth_corpus: impl BufRead,
) -> io::Result<()> {
    let (mut tpos, mut ppos, mut gpos) = (0, 0, 0);
    for (line_unseg, line_seg_groundtruth) in
        text.lines().zip(groundtruth_corpus.lines())
    {
        let line_unseg = line_unseg?;
        let line_seg_groundtruth = line_seg_groundtruth?;
        let line_seg = jieba
            .cut(&line_unseg, true)
            .into_iter()
            .map(|t| t.word)
            .collect::<Vec<_>>()
            .join(" ");
        eval::evaluate_sentence(
            &line_seg,
            &line_seg_groundtruth,
            &mut tpos,
            &mut gpos,
            &mut ppos,
        );
    }
    let precision = if ppos == 0 {
        1.0
    } else {
        tpos as f64 / ppos as f64
    };
    let recall = if gpos == 0 {
        1.0
    } else {
        tpos as f64 / gpos as f64
    };
    let f1 = eval::f_score(precision, recall, 1.0);
    let f125 = eval::f_score(precision, recall, 1.25);
    println!("f1    {}", f1);
    println!("f1.25 {}", f125);
    Ok(())
}
