comment How many times the Report evaluates each part of a for statement.
  The step-until element is defined by its expansion (4.6.4.2):
      V := A,
  L1: if (V - C) × sign(B) > 0 then go to element exhausted,
      S,
      V := V + B,
      go to L1
  (with semicolons, which cannot be written in a comment), so the limit C is evaluated once for every test, and the step B twice
  for every time round, once in the test and once in the increment.
  With ten times round there are eleven tests and ten increments;
begin
  integer na, nb, nc, nv, i, count;

  integer procedure A; begin na := na + 1; A := 1 end;
  integer procedure B; begin nb := nb + 1; B := 1 end;
  integer procedure C; begin nc := nc + 1; C := 10 end;

  count := 0;
  for i := A step B until C do count := count + 1;
  outstring(1, "body ran "); outinteger(1, count); outstring(1, "\n");
  outstring(1, "A evaluated "); outinteger(1, na); outstring(1, "\n");
  outstring(1, "B evaluated "); outinteger(1, nb); outstring(1, "\n");
  outstring(1, "C evaluated "); outinteger(1, nc); outstring(1, "\n");

  comment A limit that changes inside the loop is seen at once;
  begin
    integer n;
    n := 3; count := 0;
    for i := 1 step 1 until n do begin count := count + 1; if i = 2 then n := 5 end;
    outstring(1, "with the limit raised from 3 to 5, the body ran "); outinteger(1, count); outstring(1, "\n")
  end;

  comment A while element evaluates its expression before each test;
  count := 0;
  for i := 1, i + i while i < 100 do count := count + 1;
  outstring(1, "1, i + i while i < 100: "); outinteger(1, count);
  outstring(1, " times, i = "); outinteger(1, i); outstring(1, "\n");

  comment The parameter called by name is evaluated at each use;
  begin
    integer procedure twice(x); integer x; twice := x + x;
    integer procedure next; begin nv := nv + 1; next := nv end;
    nv := 0;
    outstring(1, "twice(next) = "); outinteger(1, twice(next));
    outstring(1, " (next was called "); outinteger(1, nv); outstring(1, "times)\n")
  end
end
