comment Each line of output tells the Revised Report from the Electrologica
  X1 system of 1962, where the two differ, as far as the documents say.
  The numbers are those of the "special properties" in the X1 user manual
  (Kruseman Aretz 2003, section 1.3). Run it with and without --x1.
  The declarations are in the order the X1 requires (property 15), and
  every formal parameter is specified (property 19), so both accept it;
begin
  integer i, seen, calls, abcdefghiX, n;
  real x;

  integer procedure stride;
  begin if calls = 0 then seen := i; calls := calls + 1; stride := 1 end;

  integer procedure note(k); value k; integer k;
  begin outinteger(1, k); note := k end;

  procedure order(a, b); value a, b; integer b; integer a; ;

  integer procedure half(y); value y; real y; half := y / 2;

  comment Property 11: the X1 evaluates the step before it assigns the
    initial value, so the first call of stride sees the old value of i;
  i := 7; calls := 0;
  for i := 1 step stride until 1 do ;
  outstring(1, "11. the step first saw i = "); outinteger(1, seen); outstring(1, "\n");

  comment Property 22: values are assigned in the order of specification;
  outstring(1, "22. value parameters evaluated in the order: ");
  order(note(1), note(2)); outstring(1, "\n");

  comment Property 18: the value of a function is not converted to the
    type it is declared with. half(3) is 1.5, rounded to 2 by the Report;
  outstring(1, "18. the integer procedure half(3) = "); outreal(1, half(3)); outstring(1, "\n");

  comment Property 3: only the first nine symbols of an identifier count,
    so abcdefghiY inside the block is the same identifier as abcdefghiX
    and hides it;
  abcdefghiX := 0;
  begin integer abcdefghiY; abcdefghiX := 1 end;
  outstring(1, "3. the outer abcdefghiX = "); outinteger(1, abcdefghiX); outstring(1, "\n");

  comment A real variable of the X1 keeps a mantissa of 40 bits, but
    1/3 is computed with 52 (X1-X8 comparison, section 3.2);
  x := 1 / 3;
  outstring(1, "3.2 x := 1/3, x - 1/3 = "); outreal(1, x - 1 / 3); outstring(1, "\n");

  comment And a sum accumulated in a variable loses the difference;
  x := 0;
  for n := 1 step 1 until 1000 do x := x + 1 / n;
  outstring(1, "3.2 the sum of 1/n to 1000 = "); outreal(1, x); outstring(1, "\n")
end
