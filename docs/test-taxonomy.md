# Taxonomy of tests

To ensure `jieba.vim` matches Vim builtin behavior as much as possible, several kinds of tests are run as part of CI.

## Unit case verification

This test targets the correctness of vimscript side in a single keymap.
The test author is supposed to provide:

- The keymap to test.
- Vim input context.
- Vim output context.
- The groundtruth model output.

The test run is split into std-run and custom-run.
In std-run, we verify that under the provided Vim input context, the keymap indeed yields the Vim output context when executed by Vim builtin mapping.
Then in custom-run, we assert that the keymap again yields the same Vim output context when executed by the plugin mapping, using the groundtruth model output without running the Rust model.

The custom-run produces a trace file in jsonl format recording the received model input and provided model output, which can then be used to test the Rust model independently.

## Bootstrap case verification

*Deprecated*

This test targets the correctness of the whole plugin in a single keymap.
The test author is supposed to provide:

- The keymap to test.
- Vim input context.

The test run is split into std-run and custom-run.
In std-run, we record the Vim output context yielded by the keymap executed with Vim builtin mapping.
Then in custom-run, we assert that the keymap executed by the plugin mapping results in exactly the same Vim output context.
If a bootstrap case verification is successful, it can be exported as a unit case verification.

## Basic integrated case verification

This is the working version of *bootstrap case verification*.

This test targets the correctness of the whole plugin in a single keymap.
The test author is supposed to provide:

- The keymap to test.
- Vim input context.

The test run is split into std-run and custom-run.
In std-run, we record the Vim output context yielded by the keymap executed with Vim builtin mapping.
Then in custom-run, we assert that the keymap executed by the plugin mapping results in exactly the same Vim output context.
The custom-run, if successfully return, will produce a trace file in jsonl format recording the received model input and the produced model output, which can then be used to test the Rust model independently.

## Integrated test

This test targets the correctness of the whole plugin under arbitrary keymaps or command execution.
The test author is supposed to provide:

- Vim input context.
- The keymaps and/or commands to run.
- Vim output context.

In the test run, we simply initialize Vim runtime according to the input context, execute the keymaps or commands, and assert that the resulting output context matches the provided output context.
