comment Exponentiation, 3.3.4.3 and 3.3.5. The operator groups from the
  left, so 2↑3↑2 is (2↑3)↑2, and a sign before an expression applies after ↑,
  so -2↑2 is -(2↑2). The type of i↑j depends on the value of j: an integer
  for j > 0, a real for j < 0;
begin
  outinteger(1, 2 ↑ 3 ↑ 2); outstring(1, "\n");
  outinteger(1, -2 ↑ 2); outstring(1, "\n");
  outinteger(1, 2 ↑ 10); outstring(1, "\n");
  outreal(1, 2 ↑ (-1)); outstring(1, "\n");
  outreal(1, 2.0 ↑ 3); outstring(1, "\n");
  outreal(1, 9 ↑ 0.5); outstring(1, "\n");
  outinteger(1, 7 ↑ 0); outstring(1, "\n");
  outreal(1, 0 ↑ 2.5); outstring(1, "\n");
  outinteger(1, 7 ÷ 2); outinteger(1, -7 ÷ 2); outinteger(1, 7 ÷ (-2)); outstring(1, "\n");
  outreal(1, 7 / 2); outstring(1, "\n");
  outinteger(1, 1 + 2 × 3 ↑ 2); outstring(1, "\n");
  outreal(1, abs(-7) / 2); outinteger(1, iabs(-7) ÷ 2); outstring(1, "\n");
  outinteger(1, entier(-2.5)); outinteger(1, sign(-0.1)); outstring(1, "\n");
  outinteger(1, 0 ↑ 0)
end
