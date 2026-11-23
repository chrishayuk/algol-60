comment Labels, switches and go to statements (3.5, 4.3).
  A label may be an unsigned integer, as in the Revised Report.
  A go to statement whose switch designator is undefined, because the
  subscript is out of range, is a dummy statement (4.3.5).
  A go to statement may enter a compound or conditional statement, but
  not a block or a for statement;
begin
  integer i, n;
  switch s := one, two, if n > 10 then big else small, 99;

  for i := 1 step 1 until 5 do
  begin
    n := i × 4;
    go to s[i];
  one: outstring(1, "one\n"); go to next;
  two: outstring(1, "two\n"); go to next;
  big: outstring(1, "big\n"); go to next;
  small: outstring(1, "small\n"); go to next;
  99: outstring(1, "ninety-nine\n"); go to next;
  next:
  end;

  comment s[7] is undefined, so this is a dummy statement;
  go to s[7];
  outstring(1, "go to s[7] did nothing\n");

  comment Into the middle of a conditional statement. After the jump the
    rest of the then-branch is done and the else-branch is not;
  n := 0;
  go to inside;
  if n = 1 then
  begin
    outstring(1, "not printed\n");
  inside: outstring(1, "inside the then-branch\n")
  end
  else outstring(1, "not printed either\n");
  outstring(1, "after the conditional\n")
end
