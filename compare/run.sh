#!/bin/sh
# Run every program in compare/probes three ways: by the Revised Report
# (algol60), as the documents say the X1 behaved (algol60 --x1), and
# through the real 1964 X1 compiler on Serge Vakulenko's simulator (x1sim,
# built in sources/sergev-x1/build). Writes compare/results/<name>.{report,x1,x1sim}.
cd "$(dirname "$0")/.."
ROOT=$PWD
SIM=$ROOT/sources/sergev-x1
export PATH="$SIM/build/compiler:$PATH"
export X1ALGOL_LIB="$SIM/library/x1algol.lib"
mkdir -p compare/results compare/obj
limit() { perl -e 'alarm shift; exec @ARGV' "$@"; }
for f in compare/probes/*.a60; do
    n=$(basename "$f" .a60)
    in=/dev/null; [ -f "compare/probes/$n.in" ] && in="$ROOT/compare/probes/$n.in"
    limit 20 target/release/algol60 "$f" < "$in" > compare/results/$n.report 2>&1
    limit 20 target/release/algol60 --x1 "$f" < "$in" > compare/results/$n.x1 2>&1
    # x1sim leaves the object program, and what the compiler typed on the
    # console, in <name>.x1 in its working directory.
    (cd compare/obj && limit 20 "$SIM/build/simulator/x1sim" "$ROOT/$f" < "$in" > "$ROOT/compare/results/$n.x1sim" 2>&1 \
        || echo "(exit $?)" >> "$ROOT/compare/results/$n.x1sim")
done
