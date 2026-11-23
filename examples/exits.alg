comment Labels passed as parameters, and go to statements that leave
  procedures. A label belongs to one activation of its block, so a jump to
  the label m passed down by the activation with n = 1 resumes that
  activation, abandoning those inside it;
begin
  procedure p(n, l);
    value n; integer n; label l;
  begin
    outstring(1, "enter "); outinteger(1, n); outstring(1, "\n");
    if n = 0 then go to l;
    p(n - 1, m);
    outstring(1, "returned to "); outinteger(1, n); outstring(1, "\n");
    go to out;
  m: outstring(1, "jumped to m of "); outinteger(1, n); outstring(1, "\n");
  out:
  end;

  comment Leaving a deep recursion at once, from a function designator in
    the middle of an expression;
  integer procedure search(k);
    value k; integer k;
  begin
    if k = 37 then go to found;
    search := search(k + 1)
  end;

  p(3, done);
done: outstring(1, "done\n");
  outinteger(1, 1 + search(1));
  outstring(1, "not printed\n");
found: outstring(1, "found 37\n")
end
