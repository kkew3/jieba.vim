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
