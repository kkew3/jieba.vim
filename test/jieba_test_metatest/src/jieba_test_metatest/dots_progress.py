# Copyright 2026 Kaiwen Wu. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

"""A simple dots progress bar."""


class DotsProgress:
    def __init__(self, flush=True):
        self.dots = 0
        self.n_dots_in_a_row = 80
        self.flush = flush

    def step(self, *, err=False):
        dot = "F" if err else "."
        print(dot, end="", flush=self.flush)
        self.dots += 1
        if self.dots % self.n_dots_in_a_row == 0:
            print(f" {self.dots}")

    def reset(self):
        if self.dots % self.n_dots_in_a_row != 0:
            print(f" {self.dots}")
        self.dots = 0

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        self.reset()
