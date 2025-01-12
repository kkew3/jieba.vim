// Copyright 2024 Kaiwen Wu. All Rights Reserved.
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

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::cases::VerifiableCase;
use super::vim_env::{VimBundlePath, VimDistro};

fn write_group_vader<I: IntoIterator<Item = P>, P: AsRef<Path>>(
    path: &Path,
    sub_vader_paths: I,
) {
    let mut f = File::create(path).unwrap();
    for path in sub_vader_paths {
        writeln!(f, "Include: {}", path.as_ref().to_str().unwrap()).unwrap();
    }
}

/// Verify all cases in the given group. Return `Err(log)` if verification
/// fails.
pub fn verify_cases<C>(
    group_name: &str,
    cases: &HashMap<String, Vec<C>>,
) -> Result<(), String>
where
    C: VerifiableCase + PartialEq + Serialize + DeserializeOwned,
{
    let basedir: PathBuf = [
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        ".verified_cases".into(),
    ]
    .iter()
    .collect();
    fs::create_dir(&basedir).ok();

    // Create the group directory if not exists.
    fs::create_dir(basedir.join(group_name)).ok();

    // Try loading verification results, and record the indices of the verified
    // cases.
    let mut verified_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (case_name, sub_cases) in cases.iter() {
        // Whether each case has been verified.
        let ind = verified_indices.entry(case_name.to_string()).or_default();
        for (i, case) in sub_cases.iter().enumerate() {
            let verified_case_path = basedir.join(format!(
                "{}/{}-{}-verified.json",
                group_name,
                case_name,
                i + 1,
            ));
            if let Ok(s) = fs::read_to_string(verified_case_path) {
                let verified_case: C = serde_json::from_str(&s).unwrap();
                if case == &verified_case {
                    ind.push(i);
                }
            }
        }
    }

    // Create a minimal vimrc if not already exists.
    let vimrc_path = basedir.join("vimrc");
    let vim_bundle_path = VimBundlePath::new_from_env();
    if let Ok(mut file) = File::create_new(vimrc_path) {
        write!(file, "set rtp+={}\n", vim_bundle_path.get_vader_rtp()).unwrap();
        write!(file, "set nocompatible\n").unwrap();
    }

    // Create the vim vader files for cases that are not verified.
    let mut case_paths = Vec::new();
    for (case_name, sub_cases) in cases.iter() {
        let ind = verified_indices.get(case_name).unwrap();
        for (i, case) in sub_cases
            .iter()
            .enumerate()
            .filter(|(i, _)| !ind.contains(i))
        {
            let case_path = basedir.join(format!(
                "{}/{}-{}.vader",
                group_name,
                case_name,
                i + 1
            ));
            case.to_vader(&case_path);
            case_paths.push(case_path);
        }
    }
    // Create the group vader file.
    let group_path = basedir.join(format!("{}.vader", group_name));
    write_group_vader(
        &group_path,
        case_paths
            .iter()
            .map(|dir| dir.strip_prefix(&basedir).unwrap()),
    );

    // Run the tests.
    let vim_bin = VimDistro::new_from_env();
    let proc = Command::new(vim_bin.as_ref())
        .args(&[
            "-u",
            "vimrc",
            "-c",
            &format!("silent Vader! {}", group_path.to_str().unwrap()),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .current_dir(&basedir)
        .spawn()
        .unwrap();
    let proc_out = proc.wait_with_output().unwrap();
    if proc_out.status.success() {
        // Write cache to disk to indicate verification success.
        for (case_name, sub_cases) in cases.iter() {
            let ind = verified_indices.get(case_name).unwrap();
            for (i, case) in sub_cases
                .iter()
                .enumerate()
                .filter(|(i, _)| !ind.contains(i))
            {
                let verified_case_path = basedir.join(format!(
                    "{}/{}-{}-verified.json",
                    group_name,
                    case_name,
                    i + 1,
                ));
                let s = serde_json::to_string(case).unwrap();
                let mut file = File::create(verified_case_path).unwrap();
                write!(file, "{}", s).unwrap();
            }
        }
        Ok(())
    } else {
        // Otherwise, return the stderr of the process.
        let stderr = String::from_utf8_lossy(&proc_out.stderr);
        Err(stderr.into())
    }
}
