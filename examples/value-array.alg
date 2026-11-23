comment An array called by value is copied at the call, so the procedure
  can change its copy without changing the actual array. Called by name,
  the array is shared;
begin
  integer array x[1:3];
  procedure byvalue(a); value a; integer array a;
  begin a[1] := 99; outstring(1, "inside, by value: "); outinteger(1, a[1]); outstring(1, "\n") end;
  procedure byname(a); integer array a;
  begin a[1] := 99; outstring(1, "inside, by name: "); outinteger(1, a[1]); outstring(1, "\n") end;

  x[1] := 1;
  byvalue(x); outstring(1, "after: "); outinteger(1, x[1]); outstring(1, "\n");
  byname(x); outstring(1, "after: "); outinteger(1, x[1]); outstring(1, "\n")
end
