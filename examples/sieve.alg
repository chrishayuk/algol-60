comment The primes below n, by the sieve of Eratosthenes. The bound of
  the array is read from the input, so it is fixed only when the block
  that declares the array is entered (5.2.4.2);
begin
  integer n;
  ininteger(0, n);
  begin
    integer i, j, count;
    Boolean array composite[2:n];
    count := 0;
    for i := 2 step 1 until n do
    begin
      if ¬composite[i] then
      begin
        count := count + 1;
        outinteger(1, i);
        for j := i × i step i until n do composite[j] := true
      end
    end;
    outstring(1, "\n"); outinteger(1, count); outstring(1, "primes\n")
  end
end
