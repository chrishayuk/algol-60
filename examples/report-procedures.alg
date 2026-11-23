comment The examples of procedure declarations in section 5.4.2 of the
  Revised Report, with a program to call them. The procedure texts are
  those of the printing in CACM 6(1), 1963, page 14, apart from the
  character set. They use the parameter delimiter
  ") letter string :(" for the comma between parameters, as in
  Spur (A) Order: (7) Result: (V);
begin
  procedure Spur(a) Order: (n) Result: (s);
    value n;
    array a; integer n; real s;
  begin integer k;
    s := 0;
    for k := 1 step 1 until n do s := s + a[k, k]
  end;

  procedure Transpose(a) Order: (n);
    value n;
    array a; integer n;
  begin real w; integer i, k;
    for i := 1 step 1 until n do
      for k := 1 + i step 1 until n do
      begin w := a[i, k];
        a[i, k] := a[k, i];
        a[k, i] := w
      end
  end Transpose;

  integer procedure Step(u); real u;
    Step := if 0 ≤ u ∧ u ≤ 1 then 1 else 0;

  procedure Absmax(a) size: (n, m) Result: (y) Subscripts: (i, k);
    comment The absolute greatest element of the matrix a, of size n by m,
      is transferred to y, and the subscripts of this element to i and k;
    array a; integer n, m, i, k; real y;
  begin integer p, q;
    y := 0;
    for p := 1 step 1 until n do
      for q := 1 step 1 until m do
        if abs(a[p, q]) > y then
        begin y := abs(a[p, q]); i := p; k := q end
  end Absmax;

  procedure Innerproduct(a, b) Order: (k, p) Result: (y);
    value k;
    integer k, p; real y, a, b;
  begin real s;
    s := 0;
    for p := 1 step 1 until k do s := s + a × b;
    y := s
  end Innerproduct;

  real array A[1:3, 1:3], B[1:3];
  integer i, j, r, c;
  real v;

  for i := 1 step 1 until 3 do
  begin
    B[i] := i;
    for j := 1 step 1 until 3 do A[i, j] := 10 × i + j
  end;
  A[2, 3] := -50;

  Spur(A) Order: (3) Result: (v);
  outstring(1, "Spur(A) = "); outreal(1, v); outstring(1, "\n");

  Absmax(A) size: (3, 3) Result: (v) Subscripts: (r, c);
  outstring(1, "Absmax: "); outreal(1, v); outstring(1, "at "); outinteger(1, r); outinteger(1, c);
  outstring(1, "\n");

  comment Jensen's device: a and b are called by name and depend on p;
  Innerproduct(A[2, j], B[j], 3, j, v);
  outstring(1, "row 2 of A times B = "); outreal(1, v); outstring(1, "\n");

  Transpose(A) Order: (3);
  outstring(1, "transposed, A[3, 2] = "); outreal(1, A[3, 2]); outstring(1, "\n");

  for i := -1, 0, 1, 2 do outinteger(1, Step(i / 2));
  outstring(1, "\n")
end
