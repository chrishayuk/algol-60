comment Knuth's "man or boy" test, Algol Bulletin 17 (July 1964), page 7,
  printed for k = 0 to 12. Knuth wrote that it "may separate the
  man-compilers from the boy-compilers", that he had no man-compiler,
  and that his conjecture from hand calculation, "probably wrong", was
  -121 for k = 10. The answer compilers found is -67. The program as
  printed is in man-or-boy-1964.alg;
begin
  integer k;
  real procedure A(k, x1, x2, x3, x4, x5);
    value k; integer k; real x1, x2, x3, x4, x5;
  begin
    real procedure B;
    begin k := k - 1;
          B := A := A(k, B, x1, x2, x3, x4)
    end;
    if k <= 0 then A := x4 + x5 else B
  end;
  for k := 0 step 1 until 12 do
  begin
    outinteger(1, k); outreal(1, A(k, 1, -1, -1, 1, 0)); outstring(1, "\n")
  end
end
