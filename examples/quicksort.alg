comment Hoare's quicksort (Algorithm 64, CACM 4, 1961, in spirit if not
  in letter), on an array called by name;
begin
  integer k, seed;
  integer array v[1:20];

  procedure quicksort(a, lo, hi);
    value lo, hi; integer lo, hi; integer array a;
  begin
    integer i, j, pivot, t;
    if lo < hi then
    begin
      pivot := a[(lo + hi) ÷ 2];
      i := lo; j := hi;
    scan:
      for i := i while a[i] < pivot do i := i + 1;
      for j := j while a[j] > pivot do j := j - 1;
      if i ≤ j then
      begin
        t := a[i]; a[i] := a[j]; a[j] := t;
        i := i + 1; j := j - 1
      end;
      if i ≤ j then go to scan;
      quicksort(a, lo, j);
      quicksort(a, i, hi)
    end
  end;

  seed := 7;
  for k := 1 step 1 until 20 do
  begin
    seed := (seed × 1103 + 12345) - (seed × 1103 + 12345) ÷ 1000 × 1000;
    v[k] := seed
  end;
  quicksort(v, 1, 20);
  for k := 1 step 1 until 20 do outinteger(1, v[k]);
  outstring(1, "\n")
end
