# Omap implementation trade-off

Date: September 13, 2026 | Author: Kaiwen Wu

中文翻译见<a href="#zh-cn">这里</a>。

## Introduction

To implement custom `omap` motions / text objects, there are three main patterns with their respective trade-offs:

- Omap by motion.
- Omap by visual selection.
- Omap by operator playback.

The following paragraphs describe the trade-offs of each.

## Omap by motion

The idea is: in operator-pending mode, we move the cursor to some position obtained by querying jieba model via an Ex command, and then `v:operator` will be applied to the range between the starting cursor position and the target position [characterwise exclusively][ex-char-exclusive].
This is the first approach documented in [`omap-info`][omap-info].

When it works, this approach is the cleanest one, in that it does not introduce any side-effects, nor does it change any Vim states unexpectedly.
However, this fails in many circumstances.
Besides not being able to implement `iw` text object, an obvious counterexample is that it's impossible to delete a word at the end of a line (where `|` denotes the cursor position):

```
foo |bar
```

Since the operation will be characterwise exclusive, and since the cursor cannot move beyond `r`, we end up deleting all but the last character:

```
foo |r
```

One might tempting to set [`'virtualedit'`][virtualedit] to `onemore` so that we are able to move the cursor one column beyond the last character in current line, solving the above issue.
But this solution introduces an unwanted side-effect of setting an option, which we will have a hard time resetting it by, e.g., `autocmd`.

Another plausible workaround is to use [forced-motion].
For instance, we will do `dv:call cursor(1, 7)<CR>` in the above foo-bar example to obliterate the word "bar".
Practical implementation will look like an [`<expr>`][map-expr] that returns `"v\<Cmd>call cursor(...)\<CR>"` on inslusive motion, `"V\<Cmd>call cursor(...)\<CR>"` on linewise motion, and `"\<Cmd>call cursor(...)\<CR>"` on exclusive motion.
This works until we'd like to [repeat] last change, since `<expr>` mapping will not be re-expanded on repeat, meaning that if last change is inclusive, so will be current change, even if it should not per context.

## Omap by visual selection

The idea is: in operator-pending mode, we enter Visual mode and select a range, and then `v:operator` will be applied to that range.
This is the second approach documented in [`omap-info`][omap-info].

The benefit of this option is that it's able to implement all motions / text objects with ease.
Meanwhile, nevertheless, it consequntially corrupts last Visual selection, invalidating [`gv`][gv], [`` `< ``][langle], [`` `> ``][rangle], etc.
As such, this approach may severely interrupt the usual workflow of many Vim users.

A possible amelioration is to restore `` `< `` and `` `> `` using `autocmd` that fires once immediately after the `omap` ends (c.f. <https://github.com/tpope/vim-repeat/issues/8#issuecomment-13951082>).
This is not perfect, though.
For example, it has been observed that undo (`u`) breaks the Visual state restoration;
there may be other failure triggers.

## Omap by operator playback

The idea is: we exit operator-pending mode, followed by a playback of the operation in Normal mode.
Looking weird at first glance, this is the cleanest method in terms of side-effects, with the largest functional coverage.
Since Normal mode is easy to handle, we are free to change whatever Vim states, as long as we revert the changes before returning from the playback.
The functional coverage is also quite large, perfectly supporting all operators but [`g@`][g-opfunc], where `g@` is supported unless it invokes [`getchar()`][getchar] (e.g. [`tpope/vim-surround`][vim-surround]).

## The trilemma

Our three desiderata form a trilemma:

1. Support any operator, including `g@` that calls `getchar()`.
2. Support any motions / text objects, where the operation range may or may not start/end at current cursor position.
3. The omap does not infavorablly introduce side-effects that change Vim states.

Omap by motion allows 1 and 3 but not 2.
Omap by visual selection supports 1 and 2 except 3.
Omap by operator playback satisfies 2 and 3 without 1.

## Conclusion

Right now, the support of `g@` is at best-effort level due to omap by operator playback.
If the `g@` operator invokes `getchar()`, the playback of that operator will be incorrect, mostly resulting in a silent failure of the operator keymap ([#145]).

In order to extensively support arbitrary `g@` for wider integration with other operator plugins, we will provide a global option that introduces omap by visual selection when `v:operator` equals `g@`.
And then, we will not be able to guarantee absence of side-effects.
This is a sensible cost.

On the other hand, omap by motion may appear preferable, but it's limited application range gives rise to stupendous maintenance burden that's not worth the gain.

---

<div id="zh-cn">以下为中文翻译：</div>

## 引言

要实现自定义的 `omap` 动作 / 文本对象，主要有三种模式，各有取舍：

- 通过动作实现 omap。
- 通过可视选择实现 omap。
- 通过重放操作符实现 omap。

下面分别介绍各自的取舍。

## 通过动作实现 omap

思路是：在操作符等待模式下，通过 Ex 命令查询 jieba 模型，得到一个目标位置并将光标移至该处，然后 `v:operator` 就会以[面向字符且不包含结束位置][ex-char-exclusive]的方式作用于光标起始位置与目标位置之间的范围。这是 [`omap-info`][omap-info] 中介绍的第一种方法。

在适用的情况下，这种方法最为干净，因为它既不引入任何副作用，也不会意外改变 Vim 的状态。然而，它在许多情况下都无法奏效。除了无法实现 `iw` 文本对象外，一个显而易见的反例是：它无法删除行尾的单词（其中 `|` 表示光标位置）：

```
foo |bar
```

由于操作是面向字符且不包含结束位置的，而光标又无法移到 `r` 之后，最终我们只能删掉除最后一个字符以外的部分：

```
foo |r
```

我们可能会想把 [`'virtualedit'`][virtualedit] 设置为 `onemore`，这样就能将光标移到当前行最后一个字符之后的一列，从而解决上述问题。但这种方案引入了修改选项这一不希望出现的副作用，而且很难通过 `autocmd` 等方式将其恢复。

另一种看似可行的变通方法是使用[强制动作类型][forced-motion]。例如，在上面的 foo-bar 示例中，我们可以执行 `dv:call cursor(1, 7)<CR>` 来彻底删除单词“bar”。实际实现可以采用 [`<expr>`][map-expr] 映射：对于包含结束位置的动作，返回 `"v\<Cmd>call cursor(...)\<CR>"`；对于面向行的动作，返回 `"V\<Cmd>call cursor(...)\<CR>"`；对于不包含结束位置的动作，返回 `"\<Cmd>call cursor(...)\<CR>"`。这种方法能正常工作，直到我们想要[重复上一次修改][repeat]：因为重复时不会重新展开 `<expr>` 映射，这意味着，如果上一次修改包含结束位置，那么当前修改也会如此，即便根据上下文不应这样处理。

## 通过可视选择实现 omap

思路是：在操作符等待模式下，进入可视模式并选中一个范围，然后 `v:operator` 就会作用于该范围。这是 [`omap-info`][omap-info] 中介绍的第二种方法。

这种方案的优点是能够轻松实现所有动作 / 文本对象。然而，它也会随之破坏上一次可视选择，使 [`gv`][gv]、[`` `< ``][langle]、[`` `> ``][rangle] 等失去原有作用。因此，这种方法可能会严重干扰许多 Vim 用户惯常的工作流程。

一种可能的改进方式是使用在 `omap` 结束后立即触发一次的 `autocmd` 来恢复 `` `< `` 和 `` `> ``（参见 <https://github.com/tpope/vim-repeat/issues/8#issuecomment-13951082>）。不过，这并不完美。例如，已经观察到撤销（`u`）会破坏可视状态的恢复；此外还可能存在其他导致恢复失败的触发条件。

## 通过重放操作符实现 omap

思路是：先退出操作符等待模式，然后在普通模式下重放该操作。乍看之下有些奇怪，但就副作用而言，这是最干净的方法，功能覆盖范围也最广。由于普通模式易于处理，我们可以自由更改 Vim 的任何状态，只要在重放结束返回之前恢复这些更改即可。它的功能覆盖范围也相当广，能够完美支持除 [`g@`][g-opfunc] 以外的所有操作符；对于 `g@`，只要它不调用 [`getchar()`][getchar]，也可以支持（调用该函数的例子包括 [`tpope/vim-surround`][vim-surround]）。

## 三难困境

我们的三个目标构成了一个三难困境：

1. 支持任何操作符，包括调用 `getchar()` 的 `g@`。
2. 支持任何动作 / 文本对象，其操作范围的起点或终点可以是当前光标位置，也可以不是。
3. omap 不会引入改变 Vim 状态的不良副作用。

通过动作实现 omap 可以满足 1 和 3，但无法满足 2。通过可视选择实现 omap 可以满足 1 和 2，但无法满足 3。通过重放操作符实现 omap 可以满足 2 和 3，但无法满足 1。

## 结论

目前，由于采用通过重放操作符实现的 omap，对 `g@` 的支持只能尽力而为。如果 `g@` 操作符调用了 `getchar()`，该操作符的重放就会出错，通常会导致操作符的按键映射静默失效（[#145]）。

为了全面支持任意 `g@` 操作符，从而更广泛地与其他操作符插件集成，我们将提供一个全局选项，使得当 `v:operator` 等于 `g@` 时，改用通过可视选择实现的 omap。在这种情况下，我们将无法保证没有副作用，尤其是作用于可视模式的副作用。这是一个合理的代价。

另一方面，通过动作实现 omap 或许看起来更可取，但其有限的适用范围会带来极其沉重的维护负担，得不偿失。



[omap-info]: https://vimhelp.org/map.txt.html#omap-info
[ex-char-exclusive]: https://vimhelp.org/motion.txt.html#exclusive
[virtualedit]: https://vimhelp.org/options.txt.html#%27virtualedit%27
[forced-motion]: https://vimhelp.org/motion.txt.html#forced-motion
[map-expr]: https://vimhelp.org/map.txt.html#%3Amap-%3Cexpr%3E
[repeat]: https://vimhelp.org/repeat.txt.html#.
[gv]: https://vimhelp.org/visual.txt.html#gv
[langle]: https://vimhelp.org/motion.txt.html#%60%3C
[rangle]: https://vimhelp.org/motion.txt.html#%60%3E
[g-opfunc]: https://vimhelp.org/map.txt.html#g%40
[getchar]: https://vimhelp.org/builtin.txt.html#getchar%28%29
[vim-surround]: https://github.com/tpope/vim-surround
[#145]: https://github.com/kkew3/jieba.vim/issues/145
