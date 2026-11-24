comment The TPK algorithm exactly as Knuth and Trabb Pardo print it, "in a
  dialect of Algol 60", on page 3 of "The Early Development of Programming
  Languages" (Stanford report STAN-CS-76-562, 1976, in docs/scans). Two
  things in it are not in the Revised Report. The heading of f gives the
  specification real t before the value part value t, against 5.4.1. And
  read and write are not procedures of the Report, which has no input or
  output, so the program stops at line 13. examples/tpk.alg is the same
  algorithm in the order of the Report with the input and output of the
  Modified Report, and compare/probes/tpk-1976.a60 runs it on the X1;
TPK: begin integer i; real y; real array a[0:10];
  real procedure f(t); real t; value t;
    f := sqrt(abs(t)) + 5 × t ↑ 3;
  for i := 0 step 1 until 10 do read(a[i]);
  for i := 10 step -1 until 0 do
  begin y := f(a[i]);
    if y > 400 then write(i, "TOO LARGE")
    else write(i, y);
  end
end.
