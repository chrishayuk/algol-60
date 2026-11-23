comment Reads numbers until a negative one, and prints their mean;
begin
  real x, sum;
  integer n;
  sum := 0; n := 0;
next:
  inreal(0, x);
  if x ≥ 0 then begin sum := sum + x; n := n + 1; go to next end;
  outinteger(1, n); outstring(1, "numbers, mean "); outreal(1, sum / n); outstring(1, "\n")
end
