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

mod verified_case;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use verified_case::{
    NamedVerifiedCasesAndMod, VerifiedCases, VerifiedCasesHeader,
};

#[proc_macro_attribute]
pub fn verified_cases(attr: TokenStream, item: TokenStream) -> TokenStream {
    let header = parse_macro_input!(attr as VerifiedCasesHeader);
    let rest = parse_macro_input!(item as NamedVerifiedCasesAndMod);
    let verified_cases = VerifiedCases::new(header, rest);
    match verified_cases.verify_and_write_tests(false) {
        Err(message) => panic!("{}", message),
        Ok(out) => out.into(),
    }
}

#[proc_macro_attribute]
pub fn verified_cases_dry_run(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let header = parse_macro_input!(attr as VerifiedCasesHeader);
    let rest = parse_macro_input!(item as NamedVerifiedCasesAndMod);
    let verified_cases = VerifiedCases::new(header, rest);
    match verified_cases.verify_and_write_tests(true) {
        Err(message) => panic!("{}", message),
        Ok(out) => out.into(),
    }
}
