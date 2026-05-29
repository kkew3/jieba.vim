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

import concurrent.futures


class FutureWrapper:
    def __init__(self, res, excp):
        self.res = res
        self.excp = excp

    def exception(self, *args, **kwargs):
        return self.excp

    def result(self, *args, **kwargs):
        return self.res


def pmap(setup_fn, runner, data, n_jobs, **kwargs):
    """
    `setup_fn`, if not None, should be a callable that takes each item in
    data as argument and returns either False if the item should be skipped
    from enqueueing, or a tuple to be bound with the result of the item. Then,
    `runner` will be called like `runner(item, *setup_data, **kwargs)`.
    """
    assert n_jobs >= 0
    if n_jobs == 0:
        for item in data:
            if setup_fn is not None:
                setup_data = setup_fn(item)
                if not setup_data:
                    continue
            else:
                setup_data = ()
            try:
                res = runner(item, *setup_data, **kwargs)
                yield (setup_data, FutureWrapper(res, excp=None))
            except Exception as excp:
                yield (setup_data, FutureWrapper(res=None, excp=excp))
    else:
        executor = concurrent.futures.ThreadPoolExecutor(n_jobs)
        try:
            fs = {}
            for item in data:
                if setup_fn is not None:
                    setup_data = setup_fn(item)
                    if not setup_data:
                        continue
                else:
                    setup_data = ()
                _fut = executor.submit(runner, item, *setup_data, **kwargs)
                fs[_fut] = setup_data
            for fut in concurrent.futures.as_completed(fs):
                yield (fs[fut], fut)
        finally:
            executor.shutdown(cancel_futures=True)  # requires python>=3.9
