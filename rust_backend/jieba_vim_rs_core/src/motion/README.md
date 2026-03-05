# Some notes on the theory of Vim motions

Vim motion is comprised of complex rules, and its documentation is far from being perfect.
Without a theoretical framework and a clean interpretation of how it works, it is very difficult to implement with as few bugs as possible.
Here we present such a framework from a sense of behavioral reverse engineering, without access to Vim's source code.

## Mode of motions

There are basically three modes of motion:

1. Normal mode: These are basic motions `w`, `b`, `e`, `ge`.
2. Visual mode:
   Three sub-modes exist, i.e., by character, by line, and by block, but they are essentially the same.
   Thus, we do not differentiate between these sub-modes.
3. Operator-pending mode:
   Under this mode, motion can be parameterized by an operator.
   For sake of clarity, we term the motions parameterized by different operators as different (sub-)motions.
   For example, `dw` is termed as a separate (sub-)motion than `cw`.

## Count of motions

Every motion is paired with a count that precedes the motion.
For operator-pending motions, the count can also be inserted between the operator and the base motion.
For example, `3w` means applying `w` three times, and `2de` means deletes till the end of the second word that follows.
In absence of a count number, it defaults to 1.

## Motion definition

Now we are prepared to define motion.

Let $B$ be the text buffer, and $P$ be the state of cursors (including begin/end of visual selection).
Then motion $m: P \rightarrow P$ is a map that updates the state of cursors given text buffer, in context of $B$.
Note that in our model of motions the text buffer remains unchanged throughout the time.
This means for operator-pending mode, the actual operation is pending, and we are actually manipulate the transient range of operation only, which is quite similar to the visual mode.
When the motion is paired with a non-unit count $k$, we put it as an upperscore: $m^k$.

Multiple motions can be performed in sequence, which can be viewed thoroughly as a motion.
We denote motion $m_1$ followed by $m_2$ with $m_2 \circ m_1$.
Two motions $m_1$ and $m_2$ are equal if $\forall p \in P$, $m_1(p) = m_2(p)$, and in this case we write $m_1 = m_2$.

When we need to name a specific motion in an equation, we will use Roman font.
For example, $\mathrm{dw}$ for `dw`.

## More notations

The set of natural numbers in this document refers to positive integers (excluding zero).

We will often encounter the need to represent a text buffer.
For clarity, we will use some special Unicode to represent whitespace characters: `␊` for newline, `·` for space, `␀` for empty buffer.
We will use `|` to denote the cursor, `<` for the beginning of visual selection, `>` for the end of visual selection.
Note that `<`/`>` is *not* the same as the `'<`/`'>` marks in Vim when we are talking about visual mode by lines.

## Markovian property of motions

We say a motion $m$ is *Markovian* if for any natural $i$ and $j$,

$$
m^i \circ m^j = m^{i+j}\,.\tag{1}
$$

Mostly, normal and visual motions are Markovian.
Nevertheless, some operator pending motions are notoriously not.

Take the following example:

```
|␊
a␊
```

(equivalent flattened view: `|␊a␊`)

Running $\mathrm{dw} \circ \mathrm{dw}$ takes us to

```
|␊
```

(equivalent flattened view: `|␊`)

In contrast, running $\mathrm{dw}^2$ yields an empty buffer:

```
|␀
```

(equivalent flattened view: `|␀`)

From the change of buffer, we know that $\mathrm{dw} \circ \mathrm{dw} \ne \mathrm{dw}^2$.

## Ordering of cursor position

For $p_1, p_2 \in P$, we say $p_1 < p_2$ if $p_2$ can be reached from $p_1$ by only going right along lines or down along virtual columns in the text buffer; vice versa.

## Forward/Backward motions

A forward motion $m$ is characterized by $\forall p \in P$, $p \le m(p)$.
A backward motion is such that $p \ge m(p)$.

## Motion failure

The failure of motion $m$ at cursor $p$ is denoted by the indicator $\mathbf{1}_F(m, p) = 0$.
Generally speaking, the following statement is true for any $p \in P$ and natural $k$:

$$
\mathbf{1}_F(m^{k}, p) = 0 \quad\Rightarrow\quad \mathbf{1}_F(m^{k+1}, p) = 0\,.\tag{2}
$$

When the count is 1, motion failure can be cleanly defined as follows:

$\mathbf{1}_F(m, p) = 0$ iff $p = \mathsf{bof}$ when $m$ is backward, or $p = \mathsf{eof}$ when $m$ is forward.
Here, bof refers to the beginning of file, and eof refers to the end of file.

## Tolerability of Markovian motions

We need one more definition of semi-failure.
Semi-failure occurs when for some $p$, $m(p) = \mathsf{emptyline}$, provided that $\mathbf{1}_F(m, p) = 1$.
We denote the occurrence of semi-failure by the indicator $\mathbf{1}_f(m, p) = 0$.
Notationally, this means the motion is by all means successful if $\mathbf{1}_f(m, p) = 1$.
Hence, this case of failure is not applicable to motion `e`, [because](https://vimhelp.org/motion.txt.html#e) it does not stop in an empty line.

For Markovian motions where $m^k = \underbrace{m \circ \cdots \circ m}_{k\ \text{applications}}$, for some $p$, we may find $\mathbf{1}_F(m^k, p)$ by evaluating the matrix:

$$
\begin{pmatrix}
\mathbf{1}_F(m, p) & \mathbf{1}_F(m, m(p)) & \dots & \mathbf{1}_F(m, m^{k-1}(p))\\
\mathbf{1}_f(m, p) & \mathbf{1}_f(m, m(p)) & \dots & \mathbf{1}_f(m, m^{k-1}(p))
\end{pmatrix}
$$

all of which can be derived from the above definitions of motion failures of unit-count motions.

# Explaination of some notable Vim motions

Okay, enough definitions.
Let's try to explain some notable Vim motion behaviors using what have been presented above.

## Operation range of operator-pending `w`

Quoted from <https://vimhelp.org/motion.txt.html#WORD>:

> Another special case: When using the "w" motion in combination with an operator and the last word *moved over* is at the end of a line, the end of that word becomes the end of the operated text, not the first word in the next line.

"Moving over the last word" obviously mean that the last word is contained in the operation range of an operator-pending `w`.
But how to know to define the containment?

We make the following claims:

1. A word (more generally a token) is contained if all its columns are contained.
2. A column is contained if:
   * the cursor starts from it or a column strictly before it and stops at another column strictly after it;
   * the cursor stops at it because its momentum vanishes.

Of course, these claims are only valid when the base motion is forwardly exclusive, like `w` in here.
If it's forwardly inclusive (e.g. `e`), it will be much easier.

Example 1: `|abcd␊` → `abc|d␊` (by `w`).

`abcd` is contained because `w` oughts to go to the beginning of the next word; the cursor stops at `d` is clearly due to being blocked by eol.

Example 2: `|a␊` → `|a␊` (by `w`).

`a` is contained because of the same argument.

Example 3: `|␊a␊` → `␊|a␊` (by `w`).

`a` is not contained because `a` *is* the beginning of the next word, and thus the cursor stops because its momentum expires.

## d-special of `dw`

Quoted from <https://vimhelp.org/change.txt.html#d-special>:

> An exception for the d{motion} command: If the motion is not linewise, the start and end of the motion are not in the same line, and there are only blanks before the start and there are no non-blanks after the end of the motion, the delete becomes linewise.  This means that the delete also removes the line of blanks that you might expect to remain.  Use the `o_v` operator to force the motion to be characterwise or remove the "z" flag from `'cpoptions'` (see `cpo-z`) to disable this peculiarity.

Observations:

- `|␊␊` → `␊|␊` (by `2dw`) is *not* a d-special.
- `|␊a␊` → `␊|a␊` (by `2dw`) is a d-special.
- `|␊a␊` → `␊|a␊` (by `de`) is a d-special.

To explain this, we add a patch that an eol (end-of-line) at eof is never contained in a motion's operation range, and that empty lines are not blanks.

In the first observation, the first eol is contained in the operaion range but the second one is not by assumption, and thus the first eol is the end of the motion.
Since the second eol forms an empty line, which is a non-blank, it violates the rule that "there are no non-blanks after the end of the motion".

In the second observation, the first eol is clearly contained in the motion.
The `a` is also contained in the same spirits as Example 2 of *Operation range of operator-pending `w`*.
Since after `a` there is only an eol that does not form an empty line, so not a non-blank, we conform to that rule.

In the third observation, the `a` is contained because `e` is inclusive; hence, the occurrence of the cursor on `a` directly entails that it's contained in the motion.
