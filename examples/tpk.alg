comment The TPK algorithm of Knuth and Trabb Pardo (1976), page 3, with two
  changes so that it is an ALGOL 60 program: the value part comes before the
  specification (5.4.1), and read and write are the Modified Report's
  inreal, outinteger, outreal and outstring. The input is the one Knuth and
  Trabb Pardo suggest for the Laning and Zierler version (p. 57),
  1 (.5) 2 (.25) 3.5 (1) 5.5, so the two runs can be compared value for
  value. f(t) = sqrt(|t|) + 5t↑3, printed for i = 10, 9, ..., 0;
TPK: begin integer i; real y; real array a[0:10];
  real procedure f(t); value t; real t;
    f := sqrt(abs(t)) + 5 × t ↑ 3;
  for i := 0 step 1 until 10 do inreal(0, a[i]);
  for i := 10 step -1 until 0 do
  begin y := f(a[i]);
    outinteger(1, i);
    if y > 400 then outstring(1, "TOO LARGE")
    else outreal(1, y);
    outstring(1, "\n")
  end
end.
