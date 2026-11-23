comment 4.2.3: in an assignment the subscript expressions of the left
  part are evaluated first, from left to right, then the expression, and
  then its value is assigned to all the left part variables;
begin
  integer i;
  integer array a[1:3];

  i := 1;
  a[i] := i := 2;
  outstring(1, "a[i] := i := 2 with i = 1 gives a[1] = "); outinteger(1, a[1]);
  outstring(1, "and i = "); outinteger(1, i); outstring(1, "\n");

  begin
    integer procedure bump; begin i := i + 1; bump := 10 × i end;
    i := 1;
    a[i] := bump;
    outstring(1, "a[i] := bump, where bump sets i to 2 and has the value 20: a[1] = ");
    outinteger(1, a[1]); outstring(1, "a[2] = "); outinteger(1, a[2]); outstring(1, "\n")
  end;

  comment A real assigned to an integer is rounded: entier(E + 0.5) (4.2.4);
  i := 2.5; outinteger(1, i);
  i := -2.5; outinteger(1, i);
  i := 2.49; outinteger(1, i);
  outstring(1, "\n")
end
