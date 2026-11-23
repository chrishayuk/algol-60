comment Own variables keep their values from one activation of their block
  to the next (5). Here an own counter in a procedure, and an own array.
  Variables that are not own start, in this implementation, at zero: the
  Report leaves their values undefined;
begin
  integer i;
  integer procedure counter;
  begin
    own integer n;
    n := n + 1;
    counter := n
  end;

  procedure remember(x);
    value x; integer x;
  begin
    own integer count;
    integer k;
    own integer array seen[1:5];
    count := count + 1;
    seen[count] := x;
    outstring(1, "seen:");
    for k := 1 step 1 until count do outinteger(1, seen[k]);
    outstring(1, "\n")
  end;

  for i := 1 step 1 until 3 do outinteger(1, counter);
  outstring(1, "\n");
  remember(10); remember(20); remember(30)
end
