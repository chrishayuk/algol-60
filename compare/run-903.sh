#!/bin/sh
# Run D. Holdsworth's test of the Report's euler (Resurrection 112) through
# the two Elliott 903 Algol systems on Andrew Herbert's simulator, as
# Herbert changed it and with each change undone. The programs are made
# from his demos/903algol/euler.txt and euler2.txt, which have no licence
# and so are not copied here. Needs sources/herbert-900 cloned and built
# (make all; it needs libpopt). Writes compare/results/euler-*.903-{5,6}.
cd "$(dirname "$0")/.."
ROOT=$PWD
SIM=$ROOT/sources/herbert-900
D=$SIM/demos/903algol
mkdir -p compare/results
# the 903 run scripts read demos/903algol/<name>.txt
sed 's/^    "switch" sss := nextterm;$/    "comment" no switch;/' $D/euler.txt > $D/c903-noswitch.txt
sed 's/"real" "procedure" eee(x); "comment" "value" x; "integer" x;/"real" "procedure" eee(x); "value" x; "integer" x;/' $D/euler.txt > $D/c903-value.txt
sed 's/fct((0))/fct(0)/; s/fct((i))/fct(i)/' $D/euler2.txt > $D/c903-unwrapped.txt
run() { # issue script program
    (cd "$SIM" && sh "$2" "$3" 2>&1 | grep -vE '^ *$|ADR') > "compare/results/euler-$4.903-$1"
}
run 5 903algolmasd.sh euler herbert
run 5 903algolmasd.sh c903-noswitch noswitch
run 5 903algolmasd.sh c903-value value
run 6 903algolajh.sh euler2 herbert
run 6 903algolajh.sh c903-value value
run 6 903algolajh.sh c903-unwrapped unwrapped
rm -f $D/c903-*.txt
