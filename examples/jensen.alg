comment Jensen's device. A parameter called by name is evaluated again
  every time it is used, in the environment of the call, so a procedure
  can be handed an expression together with the variable it depends on.
  The device is J. Jensen's, of Regnecentralen, Copenhagen;
begin
  integer i, j;
  real array a[1:3], b[1:3];
  real array m[1:3, 1:3];

  real procedure Sum(k, lo, hi, term);
    value lo, hi;
    integer k, lo, hi;
    real term;
  begin
    real s;
    s := 0;
    for k := lo step 1 until hi do
      s := s + term;
    Sum := s
  end;


  comment The harmonic number H(100);
  outstring(1, "H(100) = "); outreal(1, Sum(i, 1, 100, 1/i)); outstring(1, "\n");

  comment An inner product: the term names the subscript;
  for i := 1 step 1 until 3 do begin a[i] := i; b[i] := 10 × i end;
  outstring(1, "a.b = "); outreal(1, Sum(i, 1, 3, a[i] × b[i])); outstring(1, "\n");

  comment A double sum: Sum inside the term of Sum;
  for i := 1 step 1 until 3 do
    for j := 1 step 1 until 3 do m[i, j] := i × j;
  outstring(1, "sum of m = ");
  outreal(1, Sum(i, 1, 3, Sum(j, 1, 3, m[i, j]))); outstring(1, "\n");

  comment The trace, with the same procedure;
  outstring(1, "trace of m = "); outreal(1, Sum(i, 1, 3, m[i, i])); outstring(1, "\n");

  comment Sum of squares of the odd numbers below 20;
  outstring(1, "odd squares = ");
  outreal(1, Sum(i, 1, 10, (2 × i - 1) ↑ 2)); outstring(1, "\n");

  comment After the call, the controlled variable has been left by the
    for statement inside Sum. The Report says its value is then undefined
    (4.6.5), and here it is the value that failed the test;
  outstring(1, "i after the last call = "); outinteger(1, i); outstring(1, "\n")
end
