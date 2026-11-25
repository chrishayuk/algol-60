'comment' Example 2 of the Revised Report's examples of procedure
  declarations, the procedure RK, as printed in CACM 6(1), 1963, pages
  15 and 16. Its footnote says it "may not be optimal, nor has it
  actually been tested on a computer". It is written here with quote
  stropping, because then spaces are ignored, as in the reference
  language, and the text can stay as printed: Begin of program, set in
  roman type, is the label Beginofprogram, and end RR1ST, a misprint for
  RK1ST, is a comment. The scan's x := x3 is read as x := 23 by OCR.
  RK needs a procedure comp, which the Report describes and does not
  give. The one below is a reading of that description in decimal: the
  exponent of the largest of a, b and c is found, and the difference
  of a and b is scaled by it;
'begin'
  'real' xx, pi; 'integer' i, nfkt; 'array' y0, yy[1:2];

  'real' 'procedure' comp(a, b, c); 'value' a, b, c; 'real' a, b, c;
  'begin' 'real' m; 'integer' e;
    m := abs(a); 'if' abs(b) > m 'then' m := abs(b);
    'if' abs(c) > m 'then' m := abs(c);
    e := 'if' m = 0 'then' 0 'else' entier(ln(m)/ln(10)) + 1;
    comp := abs(a - b)/10.0↑e
  'end' comp;

  'procedure' RK(x,y,n,FKT,eps,eta,xE,yE,fi); 'value' x,y;
    'integer' n; 'Boolean' fi; 'real' x,eps,eta,xE; 'array'
    y,yE; 'procedure' FKT;
  'comment': RK integrates the system yk' = fk(x,y1,y2,...,yn)
    (k=1,2,...,n) of differential equations with the method of Runge-
    Kutta with automatic search for appropriate length of integration
    step. Parameters are: The initial values x and y[k] for x and the un-
    known functions yk(x). The order n of the system. The procedure
    FKT(x,y,n,z) which represents the system to be integrated, i.e.
    the set of functions fk. The tolerance values eps and eta which
    govern the accuracy of the numerical integration. The end of the
    integration interval xE. The output parameter yE which repre-
    sents the solution at x=xE. The Boolean variable fi, which must
    always be given the value true for an isolated or first entry into
    RK. If however the functions y must be available at several mesh-
    points x0, x1, ..., xn, then the procedure must be called repeat-
    edly (with x=xk, xE=xk+1, for k=0, 1, ..., n-1) and then the
    later calls may occur with fi=false which saves computing time.
    The input parameters of FKT must be x,y,n, the output parameter
    z represents the set of derivatives z[k]=fk(x,y[1],y[2], ..., y[n])
    for x and the actual y's. A procedure comp enters as a nonlocal
    identifier;
  'begin'
    'array' z,y1,y2,y3[1:n]; 'real' x1,x2,x3,H; 'Boolean' out;
    'integer' k,j; 'own' 'real' s,Hs;
    'procedure' RK1ST(x,y,h,xe,ye); 'real' x,h,xe; 'array'
      y,ye;
    'comment': RK1ST integrates one single RUNGE-KUTTA
      with initial values x,y[k] which yields the output
      parameters xe=x+h and ye[k], the latter being the
      solution at xe. Important: the parameters n, FKT, z
      enter RK1ST as nonlocal entities;
    'begin'
      'array' w[1:n], a[1:5]; 'integer' k,j;
      a[1] := a[2] := a[5] := h/2; a[3] := a[4] := h;
      xe := x;
      'for' k := 1 'step' 1 'until' n 'do' ye[k] := w[k] := y[k];
      'for' j := 1 'step' 1 'until' 4 'do'
      'begin'
        FKT(xe,w,n,z);
        xe := x+a[j];
        'for' k := 1 'step' 1 'until' n 'do'
        'begin'
          w[k] := y[k]+a[j]×z[k];
          ye[k] := ye[k] + a[j+1]×z[k]/3
        'end' k
      'end' j
    'end' RR1ST;
  Begin of program:
    'if' fi 'then' 'begin' H := xE-x; s := 0 'end' 'else' H := Hs;
    out := 'false';
  AA: 'if' (x+2.01×H-xE>0)≡(H>0) 'then'
    'begin' Hs := H; out := 'true'; H := (xE-x)/2
      'end' 'if';
    RK1ST(x,y,2×H,x1,y1);
  BB: RK1ST(x,y,H,x2,y2); RK1ST(x2,y2,H,x3,y3);
    'for' k := 1 'step' 1 'until' n 'do'
      'if' comp(y1[k],y3[k],eta)>eps 'then' 'go' 'to' CC;
    'comment': comp(a,b,c) is a function designator, the value
      of which is the absolute value of the difference of the
      mantissae of a and b, after the exponents of these quan-
      tities have been made equal to the largest of the exponents
      of the originally given parameters a,b,c;
    x := x3; 'if' out 'then' 'go' 'to' DD;
    'for' k := 1 'step' 1 'until' n 'do' y[k] := y3[k];
    'if' s=5 'then' 'begin' s := 0; H := 2×H 'end' 'if';
    s := s+1; 'go' 'to' AA;
  CC: H := 0.5×H; out := 'false'; x1 := x2;
    'for' k := 1 'step' 1 'until' n 'do' y1[k] := y2[k];
    'go' 'to' BB;
  DD: 'for' k := 1 'step' 1 'until' n 'do' yE[k] := y3[k]
  'end' RK;

  'comment' y' = y, which counts its calls;
  'procedure' growth(x, y, n, z); 'real' x; 'integer' n; 'array' y, z;
    'begin' nfkt := nfkt + 1; z[1] := y[1] 'end';

  'comment' y1' = y2, y2' = -y1, the harmonic oscillator, whose
    solution from y1 = 0, y2 = 1 is sin x, cos x;
  'procedure' swing(x, y, n, z); 'real' x; 'integer' n; 'array' y, z;
    'begin' nfkt := nfkt + 1; z[1] := y[2]; z[2] := -y[1] 'end';

  outstring(1, "y' = y from 0 to 1, e = ");
  nfkt := 0; y0[1] := 1;
  RK(0, y0, 1, growth, 1⏨-9, 1, 1, yy, 'true');
  outreal(1, yy[1]); outinteger(1, nfkt); outstring(1, "\n");

  outstring(1, "sin and cos at the meshpoints k pi/4\n");
  pi := 4 × arctan(1);
  nfkt := 0; y0[1] := 0; y0[2] := 1;
  'for' i := 0 'step' 1 'until' 3 'do'
  'begin'
    RK(i × pi/4, y0, 2, swing, 1⏨-9, 1, (i + 1) × pi/4, yy, i = 0);
    y0[1] := yy[1]; y0[2] := yy[2];
    outinteger(1, i + 1); outreal(1, yy[1]); outreal(1, yy[2]);
    outreal(1, sin((i + 1) × pi/4)); outreal(1, cos((i + 1) × pi/4));
    outstring(1, "\n")
  'end';
  outstring(1, "calls of FKT "); outinteger(1, nfkt); outstring(1, "\n")
'end'
