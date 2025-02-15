// Copyright 2024-2025 Kaiwen Wu. All Rights Reserved.
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

pub fn stack_merge<T, U, F>(elements: Vec<T>, mut rule_func: F) -> Vec<U>
where
    F: FnMut(Option<U>, T) -> Vec<U>,
{
    let mut stack: Vec<U> = vec![];
    for e in elements {
        let mut merged = rule_func(stack.pop(), e);
        stack.append(&mut merged);
    }
    stack
}

pub fn chain_into_vec<T, I, J>(i: I, j: J) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    J: IntoIterator<Item = T>,
{
    i.into_iter().chain(j.into_iter()).collect()
}
