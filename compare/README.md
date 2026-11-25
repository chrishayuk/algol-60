# Three runs of the same programs

Each program in `probes/` asks one question about the X1 system. `run.sh`
runs it three ways and keeps the output in `results/`:

- `.report` — `algol60`, the Revised Report. These programs use the X1's
  `print`, `PRINTTEXT` and `NLCR`, which the Report does not have, so this
  column is all errors; it is kept so that the script is the same for any
  program.
- `.x1` — `algol60 --x1`, the X1 as the documents describe it
  (`docs/x1.md`).
- `.x1sim` — the real 1964 compiler (Kruseman Aretz's Pascal version, in
  Serge Vakulenko's C translation) and Vakulenko's simulator of the X1,
  from https://github.com/sergev/x1-algol-compiler, built in
  `sources/sergev-x1/build`.

The probes are written with underlined word delimiters (`underline.py`
makes them from plain text) and `` ` ' `` string quotes, which both
programs read.

## Which half answers the question

The X1 system had two halves. The **compiler** survives, and x1sim runs
it. The **run-time complex** (the routines the object code calls for
arithmetic, for statements, procedure calls, input and output) was
printed by the Mathematical Centre as the series P(1)200, but no copy has
been found; x1sim's is Vakulenko's reconstruction, as `--x1` is this
project's. So where a result is decided by the compiler, x1sim is
evidence about the real X1. Where it is decided by the run-time, the two
columns are two independent reconstructions from the same documents.

## Results (23 November 2026; the three TPK rows 24 November)

| Probe | `--x1` | x1sim | Decided by | Reading |
| --- | --- | --- | --- | --- |
| `value-order` | 2 1 | 2 1 | compiler | Value parameters in the order of specification (property 22). Confirmed |
| `nine-symbols` | 0 | 0 | compiler | Nine symbols of an identifier count (property 3). Confirmed |
| `then-if` | b | b | compiler | `then if` accepted, `else` with the nearest `if`. Confirmed |
| `declaration-order` | p 3 | p3 | compiler | A procedure before a variable compiles and works, against property 15: the manual's rules were not the compiler's checks. Confirmed |
| `man-or-boy-unspecified` | −67 | −67 | compiler | Knuth's program as printed, with `x1` … `x5` unspecified against property 19, compiles and gives −67 |
| `numeric-label` | refused | compiler fails | compiler | `10:` makes the compiler stop with a fatal error (property 8). `--x1` now refuses it |
| `tpk-1976` | refused | compiler fails | compiler | Knuth and Trabb Pardo's TPK (1976, p. 3) with only `read` and `write` changed to the X1's. Its `TPK: begin` makes the compiler stop with 0–25, "exhausted source store": property 1 says a program is an *unlabelled* block. `--x1` now refuses it |
| `value-after-spec` | 1 1 5 | compiler accepts, run-time fails | compiler | `real t; value t;`, TPK's heading, against Report 5.4.1. The compiler takes it without a word but emits RVA and VAP ("real value array storage function frame", "value array positioning") for the scalar t, where `value t; real t;` gives TRAD. x1sim's run-time then stops, "Too many dimensions for value array"; what the X1's own did is not known. `--x1` warns and calls t by value |
| `tpk-x1` | 11 lines | 11 lines | both | TPK with the label dropped and the value part first: all eleven values agree to the 13 digits x1sim prints (216.2458286935 … 6); only the layout differs |
| `store-40-bits` | 0 | 0 | compiler | `x := 0.1; x - 0.1` is 0, because the compiler stores the constant 0.1 in the same 40-bit form as a variable (`look_for_constant` in x1algol.pas). `--x1` was corrected to match |
| `boolean-both-operands` | 2 | 2 | compiler | Both operands of `∧` and `∨` are evaluated |
| `primary-order` | 123 −5 | 123 −5 | compiler | `f(1) − f(2) × f(3)` calls f in the order written, not the order of the operators (property 7). Confirmed |
| `variable-then-call` | 2 | 2 | compiler | In `x + g`, where g sets x to 100, x is fetched before g is called |
| `subscript-order` | 7 12 | 7 12 | compiler | Subscripts evaluated left to right |
| `assign-subscript-first` | 20 0 | 20 0 | compiler | `a[i] := bump`, where bump changes i: the subscript is evaluated before the expression (Report 4.2.3) |
| `assign-multiple` | 2 0 2 | 2 0 2 | compiler | `a[i] := i := 2` with i = 1 assigns a[1], the left part subscripts first (Report 4.2.3) |
| `unary-minus` | −4 −5 6 −6 | −4 −5 6 −6 | compiler | `−a↑2` is −(a↑2); `a − b − c` and `c / a × b` go left to right |
| `switch-late` | 2 | 2 | both | A conditional entry of a switch is evaluated at the `go to`, not at the declaration |
| `name-reevaluated` | 40 | 40 | both | A name parameter `a[i]` is evaluated again each time it is used, with the i of that moment |
| `conditional-type` | 1.5 1.5 | 1.5 1.5 | both | `(if b then i else 2.5) / 2` with i = 3; `/` of two integers gives a real |
| `for-step-count` | 11 11 | 11 11 | both | The step evaluated once per time round (property 11) |
| `for-step-first` | 7 | 7 | both | The step evaluated before the initial value is assigned (property 11) |
| `for-variable-after` | 11 | 11 | run-time | The controlled variable after the loop. Kruseman Aretz's account of FOR6 and FOR7 (step added and stored before the test) gives 11 too |
| `for-variable-empty` | 5 11 | 5 11 | run-time | After `5 step 1 until 1` (body never executed) and `1 step 2 until 10`, as FOR6 and FOR7 imply |
| `function-type` | 1.5 | 1.5 | run-time | An `integer procedure` can give 1.5 (property 18) |
| `harmonic` | 5.187377517635 | 5.187377517635 | run-time | The sum of 1/n to 100 with 40-bit stores |
| `own-recursive` | 4 | 4 | run-time | One `own` variable shared by all activations |
| `zero-is-integer` | 0 | 0 | both | `0.0 ÷ 1` is allowed: zero is an integer (property 4) |
| `integer-power` | 64, 0.5 | 64, 0.5 | run-time | `2↑3↑2` and `2↑(−1)` |
| `division-truncates` | −3 −3 | −3 −3 | run-time | `÷` truncates towards zero |
| `integer-overflow` | stop 1–1 | error | run-time | `i := i + 1` with i = 67108863: the sum is real (X1 manual), and assigning it to i is stop 1–1 |
| `large-literal` | stop 1–8 | error | both | 67108864 is real, so `÷` refuses it (property 4; stop 1–8) |
| `integer-intermediate` | 67108864 134217726, stop 1–1 | error | run-time | **Disagree.** The X1 manual: an integer intermediate result beyond the capacity "is transformed automatically to real representation". Only the assignment to an integer stops |
| `real-zero` | 0 1 1, 1, .3094⏨−616 | 1 0 0, 1, 0 | run-time | **Disagree.** The X1 manual: 0 as a real is the least positive number, "the relation 'a = 0' can never have the value true for variable a of type real". `E − E = 0` holds in both |
| `divide-by-zero` | 1 | error | run-time | **Disagree.** `1 / 0` divides by the least positive number, and storing the quotient gives the greatest real (X1 manual) |
| `ln-negative` | 0 7 | error | run-time | **Disagree.** The X1 manual, point 6: "sqrt and ln operate on the absolute value of their argument"; `entier` may have an integer argument |
| `flot-two` | +.31416⏨+ 1 | +.31416⏨+ 1 | run-time | Agree. The X1's `FLOT` has a two-digit exponent (X1–X8 comparison, appendix A, note 3) |
| `empty-array` | stop 1–2 | error | run-time | Upper bound below lower |
| `abs-type` | 3 | error | run-time | **Disagree.** Property 6 says `abs` has the type of its argument; x1sim's `abs` gives a real |
| `sqrt-negative` | 2 | error | run-time | **Disagree.** Property 6 says `sqrt` takes the absolute value; x1sim's refuses |
| `round-to-integer` | 3 −3 2 | 3 −3 2 | run-time | −2.5 to an integer, −3 in both, against the Report's `entier(x + 0.5)`. Dijkstra's course calls a half undefined on the X1 (`docs/decisions.md`) |
| `switch-undefined` | error | jumps to `a` | run-time | Undefined (property 10); x1sim shows one thing it may do |

In every disagreement the manual is on the side of `--x1`: property 6 of [KA03, 1.3], and the extract of the X1 user manual
in appendix B.1 of the X1–X8 comparison, which describes the arithmetic
of the run-time complex.

## Zonneveld's outer planets

`historical/jaz164.alg` (appendix B of Kruseman Aretz 2003, from the PDF's
text), with the input data of Vakulenko's repository, through the real
compiler on x1sim: all 88 numbers printed are the same as those of
`algol60 --x1`. Before `--x1` stored real constants in 40 bits, one of
them differed in the ninth decimal.

To run it on x1sim:

    PATH=sources/sergev-x1/build/compiler:$PATH \
    X1ALGOL_LIB=sources/sergev-x1/library/x1algol.lib \
    sources/sergev-x1/build/simulator/x1sim historical/jaz164.alg < historical/jaz164.in

(x1sim wants the extension `.a60`; copy the file first.)

## The Report's euler on the Elliott 903

D. Holdsworth, "Three Experiments with Algol 60" (*Resurrection* 112,
2026), ran the Revised Report's first example procedure, `euler`, through
three preserved compilers: KDF9 Whetstone Algol ran it as printed, KDF9
Kidsgrove Algol stopped in translation (FAIL 017/20), and Elliott 903
Algol refused it. His program sums 1/n² with eps = 0.0001 and tim = 7.
Andrew Herbert's simulator (`sources/herbert-900`) has it changed for
each of two 903 systems: 16K Load and Go Algol Issue 5, and Herbert's
build from Issue 6 sources. `run-903.sh` runs those, and the same with
each change undone, into `results/euler-*.903-5` and `.903-6`.
(25 November 2026)

| Program | Issue 5 | Issue 6 | Reading |
| --- | --- | --- | --- |
| Herbert's (`euler.txt` on 5, `euler2.txt` on 6) | 1.635588, 1.644934, 1.625133 | the same | The same as `examples/report-euler.out` to the seven figures printed: 1.63558813640843, 1.64493406684823, 1.62513273362153 |
| `noswitch`: without the `switch sss := nextterm` declaring the label | the same | — | Herbert added it because a label must be declared before use (restriction 1), but here the label comes before its `goto`, so it is not needed |
| `value`: `eee(x)` with `value x`, as a Report programmer would write it | ERROR 108 at the call `euler(eee, …)` | run-time error 3 | Error 108: "An actual parameter which is a procedure has one of its parameters called by value" (restriction 13 of the manual). Issue 6 compiles it, then stops with integer overflow |
| `unwrapped`: `fct(i)` for Herbert's `fct((i))` | — | run-time error 3 | Herbert's note calls the error spurious; wrapping the actual parameter in parentheses avoids it |

So the 903's refusal comes down to one rule, the value parameter of a
procedure passed as a parameter. Under `--x1`, the Report's `euler` also
breaks the X1's order of specifications and declarations (property 15),
which the X1's compiler did not check.

To run: build the simulator (`cd sources/herbert-900 && make all`), then
`sh compare/run-903.sh`.
