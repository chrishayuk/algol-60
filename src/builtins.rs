//! The standard functions of the Revised Report (3.2.4, 3.2.5), and input
//! and output.
//!
//! The Report has no input or output. The procedures here are those of the
//! environmental block of the Modified Report (1976): `inchar`, `outchar`,
//! `length`, `outstring`, `outterminator`, `stop`, `fault`, `ininteger`,
//! `outinteger`, `inreal`, `outreal`, `maxreal`, `minreal`, `maxint`,
//! `epsilon`, and `iabs`; with IFIP's `insymbol` and `outsymbol` (1964),
//! and one extension, `outboolean`.
//!
//! As the Revised Report says, `abs` and the other functions yield real
//! values, except `sign` and `entier`, which are integer. So `abs(-3)` is
//! the real 3.0; `iabs` is the integer one.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Builtin {
    Abs,
    Iabs,
    Sign,
    Sqrt,
    Sin,
    Cos,
    Arctan,
    Ln,
    Exp,
    Entier,
    Length,
    Outstring,
    Outinteger,
    Outreal,
    Outboolean,
    Outchar,
    Outterminator,
    Inchar,
    Ininteger,
    Inreal,
    Stop,
    Fault,
    Maxreal,
    Minreal,
    Maxint,
    Epsilon,
    // The X1 system's own (Kruseman Aretz 2003, chapter 7).
    Read,
    Print,
    Tab,
    Nlcr,
    Space,
    Printtext,
    Sum,
    Flot,
    Fixt,
    Absfixt,
    Even,
}

use Builtin::*;

/// (name, builtin, number of parameters, whether it has a value)
pub const BUILTINS: &[(&str, Builtin, usize, bool)] = &[
    ("abs", Abs, 1, true),
    ("iabs", Iabs, 1, true),
    ("sign", Sign, 1, true),
    ("sqrt", Sqrt, 1, true),
    ("sin", Sin, 1, true),
    ("cos", Cos, 1, true),
    ("arctan", Arctan, 1, true),
    ("ln", Ln, 1, true),
    ("exp", Exp, 1, true),
    ("entier", Entier, 1, true),
    ("length", Length, 1, true),
    ("outstring", Outstring, 2, false),
    ("outinteger", Outinteger, 2, false),
    ("outreal", Outreal, 2, false),
    ("outboolean", Outboolean, 2, false),
    ("outchar", Outchar, 3, false),
    ("outsymbol", Outchar, 3, false),
    ("outterminator", Outterminator, 1, false),
    ("inchar", Inchar, 3, false),
    ("insymbol", Inchar, 3, false),
    ("ininteger", Ininteger, 2, false),
    ("inreal", Inreal, 2, false),
    ("stop", Stop, 0, false),
    ("fault", Fault, 2, false),
    ("maxreal", Maxreal, 0, true),
    ("minreal", Minreal, 0, true),
    ("maxint", Maxint, 0, true),
    ("epsilon", Epsilon, 0, true),
];

/// The procedures of the X1 system that its programs use without
/// declaration (the prefill of the name list, x1algol.pas), less XEEN,
/// which reads the operator's console. The Modified Report's
/// procedures are kept as well, so that the same test programs run in both
/// dialects; the X1 did not have them.
pub const X1_BUILTINS: &[(&str, Builtin, usize, bool)] = &[
    ("read", Read, 0, true),
    ("print", Print, 1, false),
    ("TAB", Tab, 0, false),
    ("NLCR", Nlcr, 0, false),
    ("SPACE", Space, 1, false),
    ("PRINTTEXT", Printtext, 1, false),
    ("SUM", Sum, 4, true),
    ("FLOT", Flot, 2, false),
    ("FIXT", Fixt, 3, false),
    ("ABSFIXT", Absfixt, 3, false),
    ("EVEN", Even, 1, true),
];

impl Builtin {
    fn entry(self) -> &'static (&'static str, Builtin, usize, bool) {
        BUILTINS.iter().chain(X1_BUILTINS).find(|b| b.1 == self).unwrap()
    }

    pub fn arity(self) -> usize {
        self.entry().2
    }

    pub fn has_value(self) -> bool {
        self.entry().3
    }

    pub fn name(self) -> &'static str {
        self.entry().0
    }

    /// On the X1 these were operators of the run-time complex, not
    /// procedures: they could not be actual parameters, and the functions
    /// among them could not be procedure statements (chapter 7; property 5
    /// of section 1.3).
    pub fn x1_operator(self) -> bool {
        use Builtin::*;
        matches!(self, Abs | Sign | Sqrt | Sin | Cos | Ln | Exp | Entier | Read | Print | Tab | Nlcr | Space)
    }
}
