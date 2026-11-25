comment Example 1 of the Revised Report's examples of procedure
  declarations, the procedure euler, as printed in CACM 6(1), 1963,
  page 15, apart from the character set. Where the scan prints
  i := n := t := 0, the HTML transcription at masswerk.at has the
  assignments to i and t separated, leaving n undefined. This is the procedure D. Holdsworth
  ran through three preserved compilers in Resurrection 112 (2026):
  KDF9 Whetstone Algol ran it as printed, Kidsgrove Algol stopped in
  translation with FAIL 017/20, and Elliott 903 Algol refused it.
  The calls below sum alternating series, convergent and divergent,
  which the comment says euler is good at;
begin
  integer calls, k; real s;

  procedure euler(fct, sum, eps, tim); value eps, tim;
    integer tim; real procedure fct; real sum, eps;
  comment euler computes the sum of fct(i) for i from zero up to
    infinity by means of a suitabley refined euler transformation. The
    summation is stopped as soon as tim times in succession the abso-
    lute value of the terms of the transformed series are found to be
    less than eps. Hence, one should provide a function fct with one
    integer argument, an upper bound eps, and an integer tim. The
    output is the sum sum. euler is particularly efficient in the case
    of a slowly convergent or divergent alternating series;
  begin integer i, k, n, t; array m[0:15]; real mn, mp, ds;
    i := n := t := 0; m[0] := fct(0); sum := m[0]/2;
  nextterm: i := i+1; mn := fct(i);
      for k := 0 step 1 until n do
        begin mp := (mn+m[k])/2; m[k] := mn;
          mn := mp end means;
      if (abs(mn)<abs(m[n])) ∧ (n<15) then
        begin ds := mn/2; n := n+1; m[n] :=
          mn end accept
      else ds := mn;
      sum := sum + ds;
      if abs(ds)<eps then t := t+1 else t := 0;
      if t<tim then go to nextterm
  end euler;

  real procedure sign(i); value i; integer i;
    sign := if i ÷ 2 × 2 = i then 1 else -1;

  comment 1 + 1/4 + 1/9 + ..., as in Holdsworth's eee;
  real procedure basel(x); value x; integer x;
    begin calls := calls + 1; basel := 1/(x+1)↑2 end;

  comment 1 - 1/2 + 1/3 - ... = ln 2;
  real procedure log2(i); value i; integer i;
    begin calls := calls + 1; log2 := sign(i)/(i+1) end;

  comment 4(1 - 1/3 + 1/5 - ...) = pi, Leibniz's series;
  real procedure leibniz(i); value i; integer i;
    begin calls := calls + 1; leibniz := 4 × sign(i)/(2×i+1) end;

  comment 1 - 1/4 + 1/9 - ... = pi^2/12, twice which is the sum of
    1/n^2, Euler's answer to the Basel problem;
  real procedure eta2(i); value i; integer i;
    begin calls := calls + 1; eta2 := sign(i)/((i+1)↑2) end;

  comment 1 - 1 + 1 - ... and 1 - 2 + 3 - ..., divergent, which the
    transformation sums to 1/2 and 1/4, the values Euler gave them;
  real procedure grandi(i); value i; integer i;
    begin calls := calls + 1; grandi := sign(i) end;
  real procedure natural(i); value i; integer i;
    begin calls := calls + 1; natural := sign(i) × (i+1) end;

  comment D. Holdsworth's test, from Andrew Herbert's copy of it for
    the 903 (github.com/andrewjherbert/900-Simulator-in-C, demos/903algol,
    euler.txt): the sum of 1/n^2, which is not alternating, with
    eps = 0.0001 and tim = 7. The transformation stops early, and
    the answer is near pi^2/6 only to about two figures, as is the sum
    of the first 50 terms printed after it;
  calls := 0; euler(basel, s, 0.0001, 7);
  outstring(1, "1+1/4+... "); outreal(1, s); outinteger(1, calls); outstring(1, "\n");
  outstring(1, "pi^2/6    "); outreal(1, (4 × arctan(1))↑2/6); outstring(1, "\n");
  s := 0; for k := 1 step 1 until 50 do s := s + 1/k↑2;
  outstring(1, "50 terms  "); outreal(1, s); outstring(1, "\n");

  calls := 0; euler(log2, s, 1⏨-10, 3);
  outstring(1, "ln 2      "); outreal(1, s); outinteger(1, calls); outstring(1, "\n");
  calls := 0; euler(leibniz, s, 1⏨-10, 3);
  outstring(1, "pi        "); outreal(1, s); outinteger(1, calls); outstring(1, "\n");
  calls := 0; euler(eta2, s, 1⏨-10, 3);
  outstring(1, "pi^2/6    "); outreal(1, 2 × s); outinteger(1, calls); outstring(1, "\n");
  calls := 0; euler(grandi, s, 1⏨-10, 3);
  outstring(1, "1-1+1-... "); outreal(1, s); outinteger(1, calls); outstring(1, "\n");
  calls := 0; euler(natural, s, 1⏨-10, 3);
  outstring(1, "1-2+3-... "); outreal(1, s); outinteger(1, calls); outstring(1, "\n")
end
