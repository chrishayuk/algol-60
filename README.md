# algol60

An implementation of **ALGOL 60**, from the *Revised Report on the
Algorithmic Language ALGOL 60* (Peter Naur, editor, 1962). It is an
interpreter in Rust, with no dependencies.

The part of ALGOL 60 that is hard to get right is not the syntax but call by
name: an actual parameter is evaluated again, in the caller's
environment, every time the procedure uses it. Here it is done the way the
Report's copy rule requires, with nothing evaluated once and kept, and
every place where the Report fixes an order of evaluation (assignment, the
for statement) follows the Report's own expansion.

```
begin
  real procedure Sum(k, lo, hi, term);
    value lo, hi; integer k, lo, hi; real term;
  begin real s;
    s := 0;
    for k := lo step 1 until hi do s := s + term;
    Sum := s
  end;
  integer i;
  outreal(1, Sum(i, 1, 100, 1/i))
end
```

```text
5.18737751763962
```

That is Jensen's device. Knuth's man or boy test (`examples/man-or-boy-1964.alg`,
as printed in 1964) gives −67, the answer compilers found; Knuth's own
hand calculation had guessed −121. `cargo test` checks the values up to
k = 20.

## Quick start

```bash
cargo build --release
target/release/algol60 examples/hello.alg
target/release/algol60 examples/man-or-boy.alg
target/release/algol60 examples/sieve.alg < examples/sieve.in
target/release/algol60 -e 'begin outinteger(1, 2 ↑ 3 ↑ 2) end'
```

```text
Usage: algol60 [OPTIONS] <FILE>
       algol60 [OPTIONS] -e <PROGRAM>
       algol60 [OPTIONS] -              (the program is read from standard input)
       algol60 --check <FILE>...
```

## What is implemented

All of the Revised Report except procedure bodies in code (5.4.6): blocks, `own`, dynamic arrays, switches
(with the rule that an undefined switch designator makes `go to` a dummy
statement), labels including unsigned integers, conditional statements and
expressions, for lists with step-until and while elements, procedures
typed and untyped, recursive, with parameters by value and by name,
procedures, arrays, labels, switches and strings as parameters,
unspecified parameters, and the `) letter string :(` parameter delimiter.

The Report's syntax is enforced where it refuses things: `if a then if b`,
`a × -b`, `a < b < c` and `if b then for ... else` are errors with a
message citing the section.

Input and output are the procedures of the Modified Report (1976):
`outstring`, `outinteger`, `outreal`, `inreal`, `ininteger`, `inchar`,
`outchar` and the rest. See [`src/builtins.rs`](src/builtins.rs).

## Writing it down

Three hardware representations are read:

- **reserved words**: `begin`, `real`, `if` in lower case or capitals.
  `≤ ≥ ≠ ¬ ∧ ∨ ⊃ ≡ × ÷ ↑ ⏨` may be written as themselves, or as `<= >= <>
  not and or impl eqv * div ** #`. `e` also marks an exponent after
  digits.
- **quote stropping**, chosen when the program contains `'begin'`:
  `'BEGIN' 'REAL' X; ...`, with spaces ignored outside strings, as in the
  reference language. See `examples/stropped.alg`.
- **underlining**, chosen when the program contains U+0332 COMBINING LOW
  LINE: `b̲e̲g̲i̲n̲ r̲e̲a̲l̲ x;`, as the reference language prints it. Spaces
  are ignored, in identifiers and numbers (`.27639 32022 50021`). See
  `historical/jaz164.alg`.

Strings are `"..."` (with `\n` for a newline), or the Report's nesting
`` `...' `` or `‘...’`, or `{...}` as some transcriptions write them.

## Choices

Where the Report leaves something undefined (initial values, the
controlled variable after a for statement, the type of a mixed
conditional expression), what is done here is recorded in
[`docs/decisions.md`](docs/decisions.md), with the questions each choice
raises for a comparison with historical compilers.

## Tests

```bash
cargo test
```

runs every program in `examples/` against its `.out` file
(`ALGOL_BLESS=1` rewrites them), and `tests/semantics.rs`, which checks the
Report's rules one at a time with programs small enough to work out by
hand.

## In the browser

`web/` is a playground that runs each program twice, as the Report
defines it and as the X1 ran it, side by side, with the lines that differ
marked. It is the same interpreter compiled to WebAssembly for WASI.
`web/wasi.js` is a small host that supplies the program, standard input and
output, so the page needs no other library.

```bash
node web/build.mjs                  # builds web/algol60.wasm and web/examples.json
node web/test.mjs                   # runs every example in it, both ways, against the .out files
python3 -m http.server -d web       # then open http://localhost:8000
```

WASI has no threads, so in the browser the interpreter cannot have the
4 GB stack it starts itself with natively. It runs on a 64 MB stack, set
in `.cargo/config.toml`. The browser's own stack is the tighter limit.
Chrome's page runs man or boy to k = 10 and its workers only to k = 9, so a
program a worker finds too deep is run again on the page, with a limit on
its steps.

## The X1 dialect

`algol60 --x1` runs a program as the Dijkstra–Zonneveld system for the
Electrologica X1, the first ALGOL 60 implementation (1960), behaved
according to its 1962 user manual and Kruseman Aretz's descriptions of
it: the for statement evaluates its step once a time round and before the
initial value, integers have 26 bits, stored reals 40, `abs` keeps the
type of its argument, and only nine characters of an identifier count.
What the X1's manual forbade but its compiler did not check is a warning.
Every rule cites its source;
see [`docs/x1.md`](docs/x1.md), which also lists what the comparison has
found so far, including a comparison with Serge Vakulenko's independent
X1 simulator. Every example has its X1 output in `examples/*.x1.out`.

The check that counts most is the X1's own output. Zonneveld's
outer-planets program was run on the X1 in 1964, and his thesis prints
the result: 82 numbers. `--x1` prints 80 of them digit for digit; the
other two differ in the last digit, where each value lies just above a
decimal tie. See [`historical/README.md`](historical/README.md).

The documents are listed in [`sources/README.md`](sources/README.md).

`historical/` holds programs from the Mathematical Centre itself.
