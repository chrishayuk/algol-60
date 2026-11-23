comment The eight queens, by backtracking, after Wirth's program in
  "Program development by stepwise refinement" (1971). Prints the first
  solution and the number of solutions;
begin
  integer i, count;
  integer array x[1:8];
  Boolean array a[1:8], b[2:16], c[-7:7];

  procedure try(i);
    value i; integer i;
  begin
    integer j, k;
    for j := 1 step 1 until 8 do begin
      if a[j] ∧ b[i + j] ∧ c[i - j] then
      begin
        x[i] := j;
        a[j] := b[i + j] := c[i - j] := false;
        if i < 8 then try(i + 1)
        else
        begin
          count := count + 1;
          if count = 1 then
          begin
            for k := 1 step 1 until 8 do outinteger(1, x[k]);
            outstring(1, "\n")
          end
        end;
        a[j] := b[i + j] := c[i - j] := true
      end
    end
  end;

  for i := 1 step 1 until 8 do a[i] := true;
  for i := 2 step 1 until 16 do b[i] := true;
  for i := -7 step 1 until 7 do c[i] := true;
  count := 0;
  try(1);
  outinteger(1, count); outstring(1, "solutions\n")
end
