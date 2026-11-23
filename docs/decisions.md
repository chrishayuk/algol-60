# Where the Report leaves a choice, and what is chosen here

The Revised Report defines a great deal exactly and leaves some things
undefined or ambiguous. Knuth listed the ambiguities in "The remaining
trouble spots in ALGOL 60" (CACM 10, 1967), and the Modified Report (1976)
settled most of them. This file records each point this implementation
had to decide, so that a disagreement with another implementation (the
Dijkstra–Zonneveld X1 compiler, the X8, Elliott, Whetstone, marst) can be
traced to a choice rather than found by surprise.

Each entry says what the Report says, what is done here, and where a test
or example shows it.

## Defined by the Report, and done as it says

| Point | Report | Here |
| --- | --- | --- |
| Call by name | 4.7.3.2, the copy rule | A thunk: the actual parameter and the frame of the call, evaluated again at every use. `tests/semantics.rs`, `examples/jensen.alg`, `examples/swap.alg` |
| The step-until element | 4.6.4.2, defined by an expansion | Done as the expansion: the limit is evaluated at every test, the step at every test and every increment. `examples/evaluation-count.alg` |
| The while element | 4.6.4.3 | `V := E` before every test |
| Order of assignment | 4.2.3: left-part subscripts, then the expression, then the assignments | As stated. `examples/assignment-order.alg` |
| Real to integer | 4.2.4: `entier(E + 0.5)` | As stated, so −2.5 becomes −2 |
| `↑` groups from the left | 3.3.5.2 | `2↑3↑2` is 64 |
| A sign before a term | 3.3.1 | `-2↑2` is −4. `a × -b` is refused |
| Type of `i↑j` | 3.3.4.3: integer for j > 0, real for j < 0 | As stated. `0↑0` and `0↑-1` are errors |
| `÷` | 3.3.4.2: integers only, `sign(a/b) × entier(abs(a/b))` | As stated: truncation towards zero. Real operands are an error |
| `/` | always real | As stated |
| Standard functions | 3.2.4: all real except `sign` and `entier` | So `abs(-3)` is real and `abs(-3) ÷ 2` is an error. `iabs` (Modified Report) is the integer one |
| Undefined switch designator | 4.3.5: `go to` becomes a dummy statement | As stated. `examples/labels.alg` |
| Jumps into compound and conditional statements | allowed | Allowed. Jumps into a for statement (4.6.6) or a block are not |
| Unsigned integer labels | 3.5.1; leading zeros do not count | `go to 10` reaches `0010:` |
| Comments after `end` | 2.3: up to `end`, `;` or `else` | As stated, with a warning when the swallowed text contains `:=` |
| Boolean operators | no short-circuit | Both operands are always evaluated |
| Procedure body that is not a block | 5.4.3: treated as a block | Its labels are local to the procedure |
| Array bounds | 5.2.4.2: evaluated on entry to the block; upper < lower means no elements | As stated |

## Left undefined or ambiguous by the Report

| Point | What is done here | Why |
| --- | --- | --- |
| Initial values of variables | Zero, 0.0 or false | The Report leaves them undefined. Most implementations of the time cleared store; a stricter mode that reports the use of an unassigned variable would be a useful addition for testing historical programs |
| The controlled variable after a for statement ends normally | The value that failed the test (`i` is 11 after `for i := 1 step 1 until 10`) | 4.6.5 says undefined. This is what the expansion leaves |
| Type of a conditional arithmetic expression whose branches differ | The type of the branch taken | Values carry their types here. The Modified Report makes it real if either branch is real; a program can tell the difference only through `÷` or output |
| Type compatibility of an actual parameter called by name | Not checked: a real actual for an integer formal is used as a real | The Report requires the types to agree (4.7.5) but implementations differed in whether they checked |
| Multiple assignment to variables of different types | Each variable gets the value converted to its own type | The Report requires the same type for all left parts. The Modified Report refuses it |
| `own` arrays with bounds that change | The bounds of the first entry are kept | Knuth's trouble spot. Many implementations required constant bounds |
| `own` variables in recursive procedures | One copy, shared by all activations | The Report gives one copy |
| Side effects of function designators | Allowed, in left-to-right order | The Report does not fix the order of evaluation of operands; left to right is the usual reading |
| Integer range | 64 bits; overflow is an error | Machine-dependent in the Report. The X1's word length is to be taken from the sources |
| Real arithmetic | IEEE double | Machine-dependent. How the X1 system did real arithmetic is to be taken from the sources |
| `outreal` format | Like C's `%.15g` | Implementation-defined |

## Not the Report's but needed

The Report has no input and output. The procedures of the Modified
Report's environmental block are provided (`outstring`, `outinteger`,
`outreal`, `outchar`, `inchar`, `ininteger`, `inreal`, `outterminator`,
`length`, `stop`, `fault`, `maxreal`, `minreal`, `maxint`, `epsilon`,
`iabs`), with IFIP's `insymbol` and `outsymbol`, and one extension,
`outboolean`. The channel number is accepted and ignored.

In strings written with `"`, `\n` is a newline and `\t` a tab, as in
marst. Strings in the Report's own quotes, `` `like this' `` or
`‘like this’`, nest and have no escapes.

`and`, `or`, `not`, `impl`, `eqv` and `div` are reserved words, as ASCII
spellings of `∧ ∨ ¬ ⊃ ≡ ÷`. So a parameter delimiter cannot be the word
`and`.

## Questions for the historical comparison

Answered from the documents (see [`x1.md`](x1.md)):

- The X1 evaluates the step of a for statement once each time round, and
  the first time before the initial value is assigned (property 11 and the
  run-time routines FOR5 to FOR7).
- `go to s[i]` with `i` out of range is undefined on the X1, not a dummy
  statement (property 10).
- `own` arrays: on the X1 their bounds must be integer numbers
  (property 17), so the question of changing bounds does not arise.
- The X1 evaluates primaries left to right (property 7), as here.

Still open, needing the run-time complex:

- Does it round or truncate when a real is assigned to an integer, and
  when a real is stored?
- Does it evaluate both operands of `∧` and `∨`?
- What is the value of the controlled variable after a for statement?
