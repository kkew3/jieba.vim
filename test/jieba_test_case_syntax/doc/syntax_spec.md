# Introduction

This document defines `jieba_test_case` format, a domain-specific language for specifying test cases of `jieba.vim`.

# Test case head conditionals

```
? !has:nvim
? version:820
```

means:

- `has("nvim")` should return 0.
- `v:version` should be `>=820`.

If any head conditional fails, all the following contents will be ignored.
The head conditionals, if any, must appear before any test case blocks.

# Global directives

## Version

```
#V {test_case_version}
```

The test case file version.
If `{test_case_version}` does not equal the parser's version, the parsing should abort immediately.

## Default

```
#{supported_local_directive} {directive_value}
```

Set the default of `{supported_local_directive}` for all the test cases below in current file.
Supported local directives are:

- Export type `X`
- Editor mode `M`
- Key sequence `K`
- Operator `O`
- Register `R`
- Count `C`
- State before `S0`
- Buffer before `B0`
- Model input `P`
- Model output `Q`

For example:

```
// Set default so that all the following test cases test for
// |d2w| in operator-pending mode.
#M o
#O d
#C 2
#K w

[..]

// Override previous default so that we test for |d3w| from
// now on.
#C 3
```

Default directives with multiple arguments can be split into multiple lines.
For example:

```
#S0 "a=foo "b=bar
```

can be written as:

```
#S0 "a=foo
#S0 "b=bar
```

Use the following to reset all defaults:

```
## {optional_text}
```

# Test case block

One or more local directives in consecutive lines define a test case block.
Blank lines separate test case blocks.
Directives with multiple arguments can be split into multiple lines, as above.
The order of different directives inside a block does not matter.

# Comments

```
// Lines starting with `//` are comments. No space is allowed
// before `//` for a line to be considered as comment.
```

# Escape sequences

Whitespace serves only readability purpose in a test case file.
When inputing whitespace or special characters in directives `K`, `S0`, `S1`, escape sequences should be used.
For example,

- `\<Space>` represents ` `.
- `\<CR>` represents return/enter.
- `\<C-v>` represents control-v.
- `\\` represents `\`.

Note that escape sequences should not be treated specially in the parser and should be fed to vim in verbatim during test verification or integration test.

# Local directives

## Export

```
X (u|i|b|bi)*
```

This defines how the test case should be exported.
If `u` is present, the test case will be exportable as unit case verification.
If `i` is present, the test case will be exportable as integrated case verification.
If `bi` is present, the test case will be exportable as basic integrated case verification.
If `b` is present, the test case will be exported as bootstrap case verification.
It's a parsing error if `X` having `u` present and in the same time `M` is `m`.
If nothing is present, the test case will simply be ignored.

Examples:

```
// Will export this test as unit case verification.
X u

// Will export this test as both unit and integrated case verifications.
X u i
```

## Editor mode

```
M (n|v|V|\<C-v>|o|m)
```

Provides a hint on the editor mode to be tested.
Explanation:

- `n`: normal mode motions.
- `v`: characterwise visual mode motions or text objects.
- `V`: linewise visual mode motions or text objects.
- `\<C-v>`: blockwise visual mode motions or text objects.
- `o`: operator-pending mode motions or text objects.
- `m`: initially normal mode but may mix different modes in the middle.

When `m` is selected, the test case can only be exported as integrated case verification.

## Key sequence

Either

```
K {motion_keys}
```

or

```
K (:{any command to execute} | {any normal! key sequence})+
```

The `normal!` key sequence or command to test for.

When the editor mode hint is not `m`, the first form must be used; otherwise, the second form can be used.
When the editor mode hint is `n`, text object motions cannot be used.

## Operator (N/A unless `M` is "o")

```
O {operator}
```

The operator to test for.

## Register (N/A unless `M` is "o"; optional otherwise)

```
R {register}
```

The register to use with `O`.
Default to `"`.

## Count (N/A if `M` is "m"; optional otherwise)

```
C {count}
```

The count to prepend before `{motion_keys}` and after `{operator}`.
If `{count}` is "0", prepend nothing.
Default to "0" if absent.

## State before

```
S0 ({option_name}={value} | {function_name}()={value} | '{mark_name}={position} | "{register_name}={value})*
```

Indicate the editor states before the key sequence.
The states will be set in the same order as specified in `S0`.

When `{option_name}={value}` is found, the option will be set as specified before the key sequence.

When `{function_name}()={value}` is found, certain setup key sequence will be executed before the key sequence such that the function will return `{value}` after setup. Supported functions and corresponding setup key sequence:

- `visualmode()={value}`: `normal! {value}\<Esc>`.

When `'{mark_name}={position}` is found, vimscript `call setpos("'{mark_name}", {position})` will be called.
`{position}` should be something like `[0,1,2,0]`, i.e. four integers separated by `,` and quoted by square brackets.

When `"{register_name}={value}` is found, vimscript `let @{register_name} = "{value}"`. If `value` contains `"`, it should be concatenated properly when transpiling to tests, e.g. `let @{register_name} = "{value1}" . '"' . "{value2}"`.

## State after

```
S1 ({option_name}=({value})? | {function_name}()=({value})? | '{mark_name}=({position})? | "{register_name}=({value})?)*
```

The syntax is the same as `S0`.

These will be checked against after the key sequence.

In bootstrap case verification and basic integrated case verification modes, `{value}` will be ignored, so can be left empty.

## Buffer before/after/output/pending

```
// buffer before, used to setup the input
B0 {buffer_expr}

// buffer after, used to run tests
B1 {buffer_expr}

// buffer pending
Bp {buffer_expr}

// buffer output
Bo {buffer_expr}
```

The buffer and several mark positions before the key sequence.

`{buffer_expr}` is a string consists of these special characters: {`·`, `┤`, `@`, `~`, `␊`, `<`, `>`, `[`, `]`, `|`, `\`}, plus other characters.
`·` is used to indicate space;
`┤` is used to indicate horizontal tab;
`@` is used to fill the columns gap after a multi-byte char (e.g. `你@@` correctly makes `你` take up 3 columns);
`~` is used to fill the visual columns after a `┤` or `␊`, optional if `virtualedit` option is empty;
`␊` is used to indicate end-of-line;
`<` is used to indicate the position of the mark `'<`;
`>` is used to indicate the position of the mark `'>`;
`[` is used to indicate the left end of selection in last visual mode (i.e. the position of the cursor after `normal! gvo`);
`]` is used to indicate the right end of selection in last visual mode (i.e. the cursor position after `normal! gvoo`);
`|` is used to indicate the cursor position;
`\` is used to indicate the curswant column (must be in the same line as `|`, an absent `\` means the curswant column equals the column of `|`, and an explicit `\` at `␊` means that the curswant column equals 2147483647, so in `|␊` curswant equals 1, and in `|\␊` curswant equals 2147483647; however, the presence of `~` after `␊` makes `\` indicate the virtual column, so in `|\␊~` curswant equals 2 rather than 2147483647);
`<` is also used to indicate the left end of operator pending range in pending buffer;
`>` is also used to indicate the right end of operator pending range in pending buffer.
`\` must appear in the same line as `|`.

The position of the position marks (one of {`|`, `\`, `<`, `>`, `[`, `]`}) is the position of the first non-mark character to its right.

Buffer pending `Bp`, if exists, must contain and only contain `<` and `>`.
Buffer before `B0` must not contain `<` or `>`.
The clean content of `Bo` must equal that of `B1`.

## Model output (N/A if `M` is "m")

```
Q ({position_mark} | {key}={value})+
```

Specify the expected model output.
The values of the included `{position_mark}` will be 4-element list of integers implied by `Bp` or `B1`.
It's a parsing error if any `{position_mark}` is present in `Q` but not specified in either `Bp` or `B1` of current test case block.

The `{position_mark}` will be mapped to the following key names and types:

- `|`: `cursor`, 4-element list of integers, or 5-element list of integers if `\` is also given as a input.
- `\`: no effect on its own; see `|`.
- `<`: `langle`, 4-element list of integers
- `>`: `rangle`, 4-element list of integers

For example:

```
Q < > | \ visualmode=\<C-v>
```

will result in the following result dictionary injected to test verification code (an illustrative example):

```vim
{
\ "langle": [0, 1, 1, 0],
\ "rangle": [0, 1, 5, 0],
\ "cursor": [0, 1, 1, 0, 0],
\ "visualmode": "\<C-v>"
\ }
```

## Autocmd event count

```
E ({event_name}={triggered_count})*
```

Indicate the expected number of times an autocmd event is triggered.
More than one events can be specified.

For example:

```
E ModeChanged=2 InsertLeave=1
```

indicate that `ModeChanged` event should be triggered twice, and `InsertLeave` event be triggered once.

# Example test case file

`this.jieba_test_case`:

```
#V 1

? !has:nvim
? version:900

H 6be7b7cd41a6d854a442dff4c3ea3eac9e3cd5f8 two words <<<
X u
M n
Q |
B0 |abc·def␊
K w
B1 abc·|def␊
// >>>

H a7b2f6e681098930f3f837cfa98d0ef99eee3f21 <<<
X u i
M V
Q < > visualmode=v
B0 a[]bc·def␊
C 1
K iw
B1 <[ab|]>c·def␊
S1 visualmode()=v
// >>>

// vim: foldmethod=marker
```

# Transpiling to tests

## To unit case verification

Unit case verification is a vimscript.

Illustrative example:

```jinja2
{# Head conditionals test #}
if has("nvim")
    execute "!echo continue"
    quit
    finish
endif

{# Define oracle model #}
function! JiebaOracleModel(...)
    return {"cursor": [0, 1, 5, 0, 5]}
endfunction

{# State before setup #}
" [..]

{# Buffer before setup #}
" [..]

{# Cursor movement #}
{%- if std_run %}
normal! w
{%- else %}
call JiebaNmap("w", 0, "JiebaOracleModel")
{%- endif %}

{# State after checks #}
" [..]

{# Buffer after checks #}
" [..]
```

## To bootstrap case verification

Knowning how to implement the Rust model correctly is a non-trivial task due to hidden states (e.g. the transient operation range of operator-pending mode), combinatorially large design space and peculiar corner cases (e.g. d-special).
However, verifying whether it aligns with the oracle (i.e. a running Vim instance) in a dichotomy sense is easy.
Given a Rust model, we may run it on random-generated or manually-written test cases and see if it aligns with the oracle; if true, we materialize the oracle's behavior in `jieba_test_case` (unparsing); else, return an error for further investigation.
If the Rust model behaves well enough, we are able to generate massive test cases automatically.

Bootstrap case verification is a vimscript.

Illustrative example:

```jinja2
{# Define oracle model #}
function! JiebaOracleModel(...)
    {# Will be replaced with other modes #}
    let g:model_output = call(function("JiebaNmapModel", a:000))
    return g:model_output
endfunction

{# State before setup #}
" [..]

{# Buffer before setup #}
" [..]

{# Cursor movement #}
{%- if std_run %}
normal! w
{%- else %}
call JiebaNmap("w", 0, "JiebaOracleModel")
{%- endif %}

{%- if std_run %}
{# State after querying #}
" [..]

{# Buffer after querying #}
let g:JiebaTestGroundtruth_cursor = json_encode(getcurpos())
" [..]

{# Buffer after echoing #}
!echo '{"cursor": ' . json_encode(getcurpos()) . '}'

mksession! Session.vim

{%- else %}

{# State after checking #}
" [..]

{# Buffer after checking #}
if getcurpos() !=# json_decode(g:JiebaTestGroundtruth_cursor)
    echoerr 1
    quit!
    finish
endif

{# Model behavior echoing #}
exe "!echo " . json_encode(g:model_output)

{%- endif %}
```

## Notes on unparsing

Unparsing a parsed `jieba_test_case`, in particular the buffer expression, is in general not possible.
To make it possible, we have to assume absence of `~` special tokens in the buffer expression.
