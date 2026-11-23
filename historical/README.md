# Programs from the Mathematical Centre

`jaz164.alg` is Zonneveld's integration of the outer planets ("JAZ164,
R743, Outer Planets"), from his thesis, as printed in appendix B of
Kruseman Aretz, *The Dijkstra–Zonneveld ALGOL 60 compiler for the
Electrologica X1* (2003), pages 313–318. It was taken from the text layer
of that PDF, with the report's substitutions undone (an opening quote for ∨, ^ for ∧,
~ for ¬, % for ⏨), the underlined words written with U+0332, and strings
in `` ` ' `` quotes, which both this implementation and x1sim read. The spaces inside
its strings follow the text layer, which may not match the print.

The report gives no input data. `jaz164.in` is the input of
`examples/outer_planets.sh` in https://github.com/sergev/x1-algol-compiler
(Apache-2.0), whose source for the numbers is not stated.

`jaz164.x1.out` is this implementation's run with `--x1`:

    algol60 --x1 jaz164.alg < jaz164.in

Run through the real 1964 compiler on sergev's X1 simulator (see
`compare/README.md`), the same file prints the same 88 numbers. The
compiler there is the original; the run-time routines are a
reconstruction, like this one, since the X1's were never published. No
output from the X1 itself is known. The layout of the numbers differs as
described in `docs/x1.md`.
