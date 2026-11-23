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
reconstruction, like this one, since the X1's were never published.

## The X1's own output, 1964

Zonneveld's thesis, *Automatic Numerical Integration* (Mathematical
Centre Tracts 8, 1964; https://ir.cwi.nl/pub/13099, pages 102–107),
prints the program as it ran on the X1, with its output for three
tolerances and for a run with the mass of Pluto set to 0. That version
punches its output with `FLOP`, `FIXP` and `ABSFIXP`, the punching
counterparts of `FLOT`, `FIXT` and `ABSFIXT` with the same layout (MR 54);
the version of 2003 types it.

`jaz164.1964.out` is the first run (eps = 10⁻⁴, pages 103–104), read from
the scan's text layer and laid out like `jaz164.x1.out`. Its 82 numbers
are those of `jaz164.x1.out` but for the last digit of two:

| | 1964 | `--x1` | Exact value here |
| --- | --- | --- | --- |
| S, x, T = 2430000.5 | 6.641455425 | 6.641455426 | 6.6414554255025 |
| U, y, T = 2431000.5 | 16.281370896 | 16.281370897 | 16.2813708965259 |

Both lie just above a tie of the ninth decimal (by 2.5⏨−12 and 2.6⏨−11,
about one unit of a 40-bit mantissa), where the outcome depends on the
binary precision of the X1's decimal conversion, which is not known. The
first is an input value, 6.64145542550 to eleven decimals. Truncating the
stored mantissa instead of rounding it would explain it, but then 5.97156957878
and .954786104043⏨−3, also input values, would print one lower than the
X1 printed them; so stores round, as the X1 manual says. `tests/x1.rs`
(`zonnevelds_output_of_1964`) checks the agreement.

The output also fixes two points of layout: `FIXT` puts the sign in the
first column (`+ 3.429474152`, `- .049534455`), and a real 0 is printed as
`+                  0` (the mass of Pluto, page 106).
