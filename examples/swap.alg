comment Why swap cannot be written with parameters called by name. With
  i = 1 and a[1] = 5, swap(i, a[i]) does t := i, then i := a[i], so i
  becomes 5, then a[i] := t, which is a[5] := 1: the subscript is
  evaluated again, after i has changed. This is the standard example of
  the trouble with call by name;
begin
  integer i;
  integer array a[1:5];

  procedure swap(x, y);
    integer x, y;
  begin
    integer t;
    t := x; x := y; y := t
  end;

  procedure show;
  begin
    integer k;
    outstring(1, "i = "); outinteger(1, i); outstring(1, " a =");
    for k := 1 step 1 until 5 do outinteger(1, a[k]);
    outstring(1, "\n")
  end;


  i := 1; a[1] := 5;
  show;
  swap(a[i], i);
  outstring(1, "after swap(a[i], i):\n"); show;

  i := 1; a[1] := 5; a[5] := 0;
  swap(i, a[i]);
  outstring(1, "after swap(i, a[i]):\n"); show
end
