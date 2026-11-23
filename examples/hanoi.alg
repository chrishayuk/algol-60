comment The towers of Hanoi. Every move for three discs, then the number
  of moves for ten;
begin
  integer moves;
  procedure hanoi(n, from, to, via, show);
    value n, from, to, via; integer n, from, to, via; Boolean show;
  begin
    if n > 0 then
    begin
      hanoi(n - 1, from, via, to, show);
      moves := moves + 1;
      if show then
      begin
        outstring(1, "disc "); outinteger(1, n); outstring(1, "from ");
        outinteger(1, from); outstring(1, "to "); outinteger(1, to); outstring(1, "\n")
      end;
      hanoi(n - 1, via, to, from, show)
    end
  end;
  moves := 0;
  hanoi(3, 1, 3, 2, true);
  moves := 0;
  hanoi(10, 1, 3, 2, false);
  outinteger(1, moves); outstring(1, "moves for ten discs\n")
end
