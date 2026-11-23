comment Knuth's test as printed in Algol Bulletin 17, page 7 (July 1964),
  from the scan in sources/knuth-man-or-boy-1964. Two changes: the
  printed outreal has one parameter, and here it is given the channel
  number the Modified Report's outreal takes, and the underlined word
  delimiters are written as reserved words. Knuth asked "What should the
  answer be?" and conjectured, by hand, 73 - 119 - 177 + 102 = -121;
begin real procedure A(k, x1, x2, x3, x4, x5);
           value k; integer k;
           begin real procedure B;
                    begin k := k - 1;
                          B := A := A(k, B, x1, x2, x3, x4)
                    end;
                 if k ≤ 0 then A := x4 + x5 else B
           end;
      outreal(1, A(10, 1, -1, -1, 1, 0))
end;
