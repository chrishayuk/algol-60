//! From the syntax tree to code the interpreter runs: every identifier is
//! resolved to its declaration (by static distance and slot), and every
//! statement list is flattened into instructions with jumps, so that a
//! go to statement can enter a compound or conditional statement, as the
//! Report allows, but not a block or a for statement, which it does not.

use crate::ast::*;
use crate::builtins::{Builtin, BUILTINS, X1_BUILTINS};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

pub type LabelId = u32;

/// Which ALGOL 60: the Revised Report's, or the language as the
/// Dijkstra–Zonneveld system for the Electrologica X1 implemented it, as
/// far as the documents describe it (see docs/x1.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dialect {
    #[default]
    Report,
    X1,
}

/// The largest integer of the X1: 26 bits and a sign (stop 1–1).
pub const X1_MAXINT: i64 = 67_108_863;

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Simple(Type),
    Array,
    /// A formal parameter. For one called by value and specified simple,
    /// the type assignments to it are converted to.
    Formal(Option<Type>),
}

#[derive(Clone, Copy, Debug)]
pub enum What {
    Slot(u16, Kind),
    Own(u32, Kind),
    Proc(u32),
    Switch(u32),
    Builtin(Builtin),
}

#[derive(Clone, Debug)]
pub struct Var {
    pub hops: u16,
    pub what: What,
    pub name: Rc<str>,
}

pub type Arg = Rc<Ex>;

pub enum Ex {
    Const(Value),
    Load(Var),
    Label { hops: u16, id: LabelId },
    Index(Var, Vec<Ex>),
    Call(Var, Vec<Arg>),
    Neg(Box<Ex>),
    Pos(Box<Ex>),
    Not(Box<Ex>),
    Bin(BOp, Box<Ex>, Box<Ex>),
    If(Box<Ex>, Box<Ex>, Box<Ex>),
}

pub enum Target {
    Var(Var),
    Index(Var, Vec<Ex>),
    /// The value of the procedure whose body this is, `hops` frames out.
    Result { hops: u16, ty: Type },
}

pub enum Ins {
    Assign(Vec<Target>, Ex),
    Call(Var, Vec<Arg>),
    Goto(Ex),
    JumpUnless(Ex, usize),
    Jump(usize),
    Block(Box<BlockCode>),
    For(Box<ForCode>),
}

#[derive(Default)]
pub struct Code {
    pub ins: Vec<(Ins, u32)>,
    pub labels: Vec<(LabelId, usize)>,
}

impl Code {
    pub fn find(&self, id: LabelId) -> Option<usize> {
        self.labels.iter().find(|(l, _)| *l == id).map(|(_, p)| *p)
    }
}

pub enum ArrPlace {
    Slot(u16),
    Own(u32),
}

pub enum Init {
    Simple { slot: u16, ty: Type },
    Array { place: ArrPlace, ty: Type, bounds: Rc<Vec<(Ex, Ex)>>, name: Rc<str> },
}

pub struct BlockCode {
    pub nslots: usize,
    pub inits: Vec<Init>,
    pub body: Code,
}

pub struct ParamCode {
    pub name: Rc<str>,
    pub by_value: bool,
    pub spec: Option<Spec>,
}

pub struct ProcCode {
    pub name: Rc<str>,
    pub ty: Option<Type>,
    pub params: Vec<ParamCode>,
    pub nslots: usize,
    pub body: Code,
    /// The order in which the parameters called by value are evaluated:
    /// left to right, or on the X1 the order of the specifications
    /// (property 22).
    pub value_order: Vec<usize>,
}

pub enum ForEl {
    Expr(Ex),
    Step(Ex, Ex, Ex),
    While(Ex, Ex),
}

pub struct ForCode {
    pub var: Target,
    pub var_ex: Ex,
    pub elems: Vec<ForEl>,
    pub body: Code,
}

pub struct Program {
    pub main: BlockCode,
    pub procs: Vec<ProcCode>,
    pub switches: Vec<Vec<Ex>>,
    pub owns: Vec<Value>,
    pub labels: Vec<String>,
    pub warnings: Vec<String>,
    pub dialect: Dialect,
}

#[derive(Clone)]
enum Binding {
    Slot(u16, Kind),
    Own(u32, Kind),
    Proc { id: u32, ty: Option<Type>, nparams: usize },
    Switch(u32),
    Builtin(Builtin),
    Label(LabelId),
}

struct Scope {
    names: HashMap<String, Binding>,
    nslots: u16,
    proc_id: Option<u32>,
}

struct Compiler {
    scopes: Vec<Scope>,
    procs: Vec<Option<ProcCode>>,
    switches: Vec<Option<Vec<Ex>>>,
    owns: Vec<Value>,
    labels: Vec<String>,
    dialect: Dialect,
    warnings: Vec<String>,
}

type CResult<T> = Result<T, String>;

fn err<T>(line: u32, msg: impl AsRef<str>) -> CResult<T> {
    Err(format!("line {}: {}", line, msg.as_ref()))
}

/// The X1's manual forbade it: `property` is the number in the list of
/// "special properties of the MC-Algol-system" from the user manual of 1962
/// (Kruseman Aretz 2003, section 1.3). The compiler checked almost none of
/// these (ibid.: "the almost complete absence of syntax checks"), and
/// programs that break them compile with the real compiler, so they are
/// warnings: what the X1 then did is not known.
fn x1_warning(line: u32, property: u32, msg: impl AsRef<str>) -> String {
    format!(
        "line {}: the X1 manual forbids this (property {}): {}; its compiler did not check, and what followed is not known",
        line,
        property,
        msg.as_ref()
    )
}

pub fn compile(src: &str, dialect: Dialect) -> CResult<Program> {
    let lexed = crate::lexer::lex(src)?;
    let (stmt, notes) = crate::parser::parse_program(lexed.tokens, dialect == Dialect::X1)?;
    let mut warnings = lexed.warnings;
    if dialect == Dialect::X1 {
        for line in notes.letter_delimiters {
            warnings.push(x1_warning(line, 13, "only a comma is permitted as parameter delimiter"));
        }
    }
    for line in notes.then_ifs {
        warnings.push(format!(
            "line {}: 'if' after 'then' is not allowed by the Report (4.5.1); the X1 compiler accepted it, and here 'else' goes with the nearest 'if'",
            line
        ));
    }
    let mut c = Compiler {
        warnings,
        scopes: vec![Scope { names: HashMap::new(), nslots: 0, proc_id: None }],
        procs: vec![],
        switches: vec![],
        owns: vec![],
        labels: vec![],
        dialect,
    };
    let library: Vec<_> = match dialect {
        Dialect::Report => BUILTINS.iter().collect(),
        Dialect::X1 => BUILTINS.iter().chain(X1_BUILTINS).collect(),
    };
    for (name, b, _, _) in library {
        let k = c.key(name);
        c.scopes[0].names.insert(k, Binding::Builtin(*b));
    }
    let main = c.block(&[], std::slice::from_ref(&stmt), None)?;
    Ok(Program {
        main,
        procs: c.procs.into_iter().map(|p| p.unwrap()).collect(),
        switches: c.switches.into_iter().map(|s| s.unwrap()).collect(),
        owns: c.owns,
        labels: c.labels,
        warnings: c.warnings,
        dialect,
    })
}

/// The labels belonging to the block whose statements these are: those
/// inside compound, conditional and for statements, but not inside an
/// inner block or a procedure body.
fn collect_labels(stmts: &[Stmt], out: &mut Vec<(String, u32)>) {
    for s in stmts {
        out.extend(s.labels.iter().cloned());
        match &s.kind {
            SK::Compound(v) => collect_labels(v, out),
            SK::If(_, a, b) => {
                collect_labels(std::slice::from_ref(a), out);
                if let Some(b) = b {
                    collect_labels(std::slice::from_ref(b), out);
                }
            }
            SK::For(_, _, body) => collect_labels(std::slice::from_ref(body), out),
            _ => {}
        }
    }
}

impl Compiler {
    fn warn_x1(&mut self, line: u32, property: u32, msg: impl AsRef<str>) {
        self.warnings.push(x1_warning(line, property, msg));
    }

    fn x1(&self) -> bool {
        self.dialect == Dialect::X1
    }

    /// The part of an identifier that counts: on the X1 only the first nine
    /// symbols (property 3).
    fn key(&self, name: &str) -> String {
        if self.x1() {
            name.chars().take(9).collect()
        } else {
            name.to_string()
        }
    }

    fn declare(&mut self, name: &str, line: u32, b: Binding) -> CResult<()> {
        let k = self.key(name);
        let scope = self.scopes.last_mut().unwrap();
        if scope.names.insert(k, b).is_some() {
            return err(line, format!("{} is declared twice in the same block", name));
        }
        Ok(())
    }

    fn slot(&mut self) -> u16 {
        let s = self.scopes.last_mut().unwrap();
        let n = s.nslots;
        s.nslots += 1;
        n
    }

    fn lookup(&self, name: &str, line: u32) -> CResult<(u16, Binding)> {
        let key = self.key(name);
        for (hops, scope) in self.scopes.iter().rev().enumerate() {
            if let Some(b) = scope.names.get(&key) {
                return Ok((hops as u16, b.clone()));
            }
        }
        // Programs written all in capitals call OUTREAL and the rest.
        if name == name.to_uppercase() {
            if let Some(b @ Binding::Builtin(_)) = self.scopes[0].names.get(&self.key(&name.to_lowercase())) {
                return Ok((self.scopes.len() as u16 - 1, b.clone()));
            }
        }
        err(line, format!("{} is not declared", name))
    }

    fn var(name: &str, hops: u16, b: &Binding) -> Var {
        let what = match *b {
            Binding::Slot(s, k) => What::Slot(s, k),
            Binding::Own(i, k) => What::Own(i, k),
            Binding::Proc { id, .. } => What::Proc(id),
            Binding::Switch(id) => What::Switch(id),
            Binding::Builtin(b) => What::Builtin(b),
            Binding::Label(_) => unreachable!(),
        };
        Var { hops, what, name: name.into() }
    }

    fn declare_labels(&mut self, stmts: &[Stmt]) -> CResult<()> {
        let mut labels = Vec::new();
        collect_labels(stmts, &mut labels);
        for (name, line) in labels {
            let id = self.labels.len() as LabelId;
            self.labels.push(name.clone());
            if self.x1() && name.starts_with(|c: char| c.is_ascii_digit()) {
                // Unlike most of the manual's rules, this one the compiler
                // enforces, by failing: the 1964 compiler stops with a fatal
                // error on `10:` (compare/probes/numeric-label.a60, run on
                // sergev's x1sim).
                return err(
                    line,
                    "the X1 system does not accept labels beginning with a digit (property 8 of its manual; its compiler fails on them)",
                );
            }
            if self.scopes.last().unwrap().names.contains_key(&self.key(&name)) {
                return err(line, format!("{} is declared twice in the same block (once as a label)", name));
            }
            self.declare(&name, line, Binding::Label(id))?;
        }
        Ok(())
    }

    fn block(&mut self, decls: &[Decl], body: &[Stmt], proc_id: Option<u32>) -> CResult<BlockCode> {
        self.scopes.push(Scope { names: HashMap::new(), nslots: 0, proc_id });
        let mut inits = Vec::new();
        let mut pending_arrays = Vec::new();
        let mut pending_procs = Vec::new();
        let mut pending_switches = Vec::new();
        // On the X1, scalars, then arrays, then switches, then procedures
        // (property 15).
        let mut rank = 0;
        let outermost = self.scopes.len() == 3;
        for d in decls {
            if self.x1() {
                let (r, line) = match d {
                    Decl::Simple { names, .. } => (1, names[0].1),
                    Decl::Array { segs, .. } => (2, segs[0].0[0].1),
                    Decl::Switch { name, .. } => (3, name.1),
                    Decl::Proc(p) => (4, p.name.1),
                };
                if r < rank {
                    self.warn_x1(line, 15, "declarations must come in the order: scalars, arrays, switches, procedures");
                }
                rank = rank.max(r);
                if let Decl::Array { own, segs, .. } = d {
                    if *own || outermost {
                        let constant = |e: &E| match e {
                            E::Int(_) => true,
                            E::Neg(x) | E::Pos(x) => matches!(**x, E::Int(_)),
                            _ => false,
                        };
                        for (names, bounds) in segs {
                            if !bounds.iter().all(|(lo, hi)| constant(lo) && constant(hi)) {
                                self.warn_x1(
                                    names[0].1,
                                    17,
                                    "the bounds of an own array, or of an array in the outermost block, must be integer numbers",
                                );
                            }
                        }
                    }
                }
            }
            match d {
                Decl::Simple { own, ty, names } => {
                    for (n, line) in names {
                        if *own {
                            let idx = self.owns.len() as u32;
                            self.owns.push(Value::zero(*ty));
                            self.declare(n, *line, Binding::Own(idx, Kind::Simple(*ty)))?;
                        } else {
                            let slot = self.slot();
                            self.declare(n, *line, Binding::Slot(slot, Kind::Simple(*ty)))?;
                            inits.push(Init::Simple { slot, ty: *ty });
                        }
                    }
                }
                Decl::Array { own, ty, segs } => {
                    for (names, bounds) in segs {
                        let mut places = Vec::new();
                        for (n, line) in names {
                            let place = if *own {
                                let idx = self.owns.len() as u32;
                                self.owns.push(Value::Undef);
                                self.declare(n, *line, Binding::Own(idx, Kind::Array))?;
                                ArrPlace::Own(idx)
                            } else {
                                let slot = self.slot();
                                self.declare(n, *line, Binding::Slot(slot, Kind::Array))?;
                                ArrPlace::Slot(slot)
                            };
                            places.push((place, n.clone()));
                        }
                        pending_arrays.push((*ty, places, bounds));
                    }
                }
                Decl::Switch { name, elems } => {
                    let id = self.switches.len() as u32;
                    self.switches.push(None);
                    self.declare(&name.0, name.1, Binding::Switch(id))?;
                    pending_switches.push((id, elems));
                }
                Decl::Proc(p) => {
                    let id = self.procs.len() as u32;
                    self.procs.push(None);
                    self.declare(&p.name.0, p.name.1, Binding::Proc { id, ty: p.ty, nparams: p.params.len() })?;
                    pending_procs.push((id, p));
                }
            }
        }
        self.declare_labels(body)?;
        for (ty, places, bounds) in pending_arrays {
            let mut bs = Vec::new();
            for (lo, hi) in bounds {
                bs.push((self.expr(lo, false)?, self.expr(hi, false)?));
            }
            let bs = Rc::new(bs);
            for (place, name) in places {
                inits.push(Init::Array { place, ty, bounds: bs.clone(), name: name.into() });
            }
        }
        for (id, elems) in pending_switches {
            let mut v = Vec::new();
            for e in elems {
                v.push(self.desig(e)?);
            }
            self.switches[id as usize] = Some(v);
        }
        for (id, p) in pending_procs {
            let code = self.procedure(id, p)?;
            self.procs[id as usize] = Some(code);
        }
        let mut code = Code::default();
        for s in body {
            self.stmt(s, &mut code)?;
        }
        let scope = self.scopes.pop().unwrap();
        Ok(BlockCode { nslots: scope.nslots as usize, inits, body: code })
    }

    fn procedure(&mut self, id: u32, p: &ProcDecl) -> CResult<ProcCode> {
        let mut specs: HashMap<&str, Spec> = HashMap::new();
        for (spec, names) in &p.specs {
            for (n, l) in names {
                if !p.params.iter().any(|(q, _)| q == n) {
                    return err(*l, format!("{} is specified but is not a parameter of {}", n, p.name.0));
                }
                if specs.insert(n, *spec).is_some() {
                    return err(*l, format!("{} is specified twice", n));
                }
            }
        }
        for (n, l) in &p.values {
            if !p.params.iter().any(|(q, _)| q == n) {
                return err(*l, format!("{} is in the value part but is not a parameter of {}", n, p.name.0));
            }
            if let Some(s @ (Spec::Proc(_) | Spec::Switch)) = specs.get(n.as_str()) {
                return err(*l, format!("{} is a {} and cannot be called by value (4.7.3.1)", n, s.name()));
            }
            if self.x1() && specs.get(n.as_str()) == Some(&Spec::Label) {
                self.warn_x1(*l, 21, format!("{} is specified label and cannot be in the value list", n));
            }
        }
        if self.x1() {
            for (n, l) in &p.params {
                if !specs.contains_key(n.as_str()) {
                    self.warn_x1(*l, 19, format!("{} has no specification; every formal parameter needs one", n));
                }
            }
            let mut rank = 0;
            for (spec, names) in &p.specs {
                let r = match spec {
                    Spec::Simple(_) | Spec::Str => 1,
                    Spec::Array(_) => 2,
                    Spec::Label | Spec::Switch => 3,
                    Spec::Proc(_) => 4,
                };
                if r < rank {
                    self.warn_x1(
                        names[0].1,
                        15,
                        "specifications must come in the order: scalars and strings, arrays, labels and switches, procedures",
                    );
                }
                rank = rank.max(r);
            }
        }
        // Left to right, or on the X1 in the order of specification.
        let mut value_order: Vec<usize> =
            (0..p.params.len()).filter(|&i| p.values.iter().any(|(q, _)| *q == p.params[i].0)).collect();
        if self.x1() {
            let position = |i: &usize| {
                p.specs.iter().flat_map(|(_, names)| names).position(|(n, _)| *n == p.params[*i].0)
            };
            value_order.sort_by_key(position);
        }
        self.scopes.push(Scope { names: HashMap::new(), nslots: 1, proc_id: Some(id) });
        let mut params = Vec::new();
        for (n, l) in &p.params {
            let by_value = p.values.iter().any(|(q, _)| q == n);
            let spec = specs.get(n.as_str()).copied();
            let conv = match (by_value, spec) {
                (true, Some(Spec::Simple(t))) => Some(t),
                _ => None,
            };
            let slot = self.slot();
            self.declare(n, *l, Binding::Slot(slot, Kind::Formal(conv)))?;
            params.push(ParamCode { name: n.as_str().into(), by_value, spec });
        }
        // A body that is not a block behaves as if it were one (5.4.3), so
        // its labels are local to the procedure.
        self.declare_labels(std::slice::from_ref(&p.body))?;
        let mut body = Code::default();
        self.stmt(&p.body, &mut body)?;
        let scope = self.scopes.pop().unwrap();
        Ok(ProcCode { name: p.name.0.as_str().into(), ty: p.ty, params, nslots: scope.nslots as usize, body, value_order })
    }

    fn stmt(&mut self, s: &Stmt, code: &mut Code) -> CResult<()> {
        for (name, line) in &s.labels {
            match self.lookup(name, *line)? {
                (0, Binding::Label(id)) => code.labels.push((id, code.ins.len())),
                _ => unreachable!("label {} not declared in its block", name),
            }
        }
        let line = s.line;
        match &s.kind {
            SK::Dummy => {}
            SK::Assign(lefts, e) => {
                let mut targets = Vec::new();
                for l in lefts {
                    targets.push(self.target(l)?);
                }
                let ex = self.expr(e, false)?;
                code.ins.push((Ins::Assign(targets, ex), line));
            }
            SK::Call(name, args, l) => {
                let (hops, b) = self.lookup(name, *l)?;
                match b {
                    Binding::Proc { nparams, .. } if nparams != args.len() => {
                        return err(*l, format!("{} has {} parameters, but is given {}", name, nparams, args.len()))
                    }
                    Binding::Builtin(bi) if bi.arity() != args.len() => {
                        return err(*l, format!("{} has {} parameters, but is given {}", name, bi.arity(), args.len()))
                    }
                    Binding::Builtin(bi) if self.x1() && bi.x1_operator() && bi.has_value() => {
                        self.warn_x1(*l, 5, format!("the standard function {} may not be called as a statement", name))
                    }
                    Binding::Proc { .. } | Binding::Builtin(_) | Binding::Slot(_, Kind::Formal(_)) => {}
                    _ => return err(*l, format!("{} is not a procedure", name)),
                }
                let var = Self::var(name, hops, &b);
                let args = self.actuals(args)?;
                code.ins.push((Ins::Call(var, args), line));
            }
            SK::Goto(e) => {
                let d = self.desig(e)?;
                code.ins.push((Ins::Goto(d), line));
            }
            SK::If(c, a, b) => {
                let cond = self.expr(c, false)?;
                let at = code.ins.len();
                code.ins.push((Ins::JumpUnless(cond, 0), line));
                self.stmt(a, code)?;
                if let Some(b) = b {
                    let jump_at = code.ins.len();
                    code.ins.push((Ins::Jump(0), line));
                    let else_pc = code.ins.len();
                    code.ins[at].0 = match std::mem::replace(&mut code.ins[at].0, Ins::Jump(0)) {
                        Ins::JumpUnless(c, _) => Ins::JumpUnless(c, else_pc),
                        _ => unreachable!(),
                    };
                    self.stmt(b, code)?;
                    let end = code.ins.len();
                    code.ins[jump_at].0 = Ins::Jump(end);
                } else {
                    let end = code.ins.len();
                    code.ins[at].0 = match std::mem::replace(&mut code.ins[at].0, Ins::Jump(0)) {
                        Ins::JumpUnless(c, _) => Ins::JumpUnless(c, end),
                        _ => unreachable!(),
                    };
                }
            }
            SK::For(v, elems, body) => {
                if self.x1() && matches!(body.kind, SK::If(..)) {
                    self.warn_x1(body.line, 12, "a conditional statement may not follow 'do'");
                }
                let var = self.target(v)?;
                if matches!(var, Target::Result { .. }) {
                    return err(line, "the controlled variable of a for statement must be a variable");
                }
                let var_ex = self.expr(v, false)?;
                let mut els = Vec::new();
                for el in elems {
                    els.push(match el {
                        ForElem::Expr(e) => ForEl::Expr(self.expr(e, false)?),
                        ForElem::Step(a, b, c) => {
                            ForEl::Step(self.expr(a, false)?, self.expr(b, false)?, self.expr(c, false)?)
                        }
                        ForElem::While(e, f) => ForEl::While(self.expr(e, false)?, self.expr(f, false)?),
                    });
                }
                let mut body_code = Code::default();
                self.stmt(body, &mut body_code)?;
                code.ins.push((Ins::For(Box::new(ForCode { var, var_ex, elems: els, body: body_code })), line));
            }
            SK::Compound(v) => {
                for s in v {
                    self.stmt(s, code)?;
                }
            }
            SK::Block(b) => {
                let bc = self.block(&b.decls, &b.body, None)?;
                code.ins.push((Ins::Block(Box::new(bc)), line));
            }
        }
        Ok(())
    }

    fn target(&mut self, e: &E) -> CResult<Target> {
        match e {
            E::Id(name, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Slot(_, Kind::Simple(_) | Kind::Formal(_)) | Binding::Own(_, Kind::Simple(_) | Kind::Formal(_)) => {
                        Ok(Target::Var(Self::var(name, hops, &b)))
                    }
                    Binding::Proc { id, ty: Some(ty), .. } => {
                        // The procedure's own value: only inside its body.
                        for (h, scope) in self.scopes.iter().rev().enumerate() {
                            if scope.proc_id == Some(id) {
                                return Ok(Target::Result { hops: h as u16, ty });
                            }
                        }
                        err(*line, format!("{} can only be assigned to inside the body of {} (5.4.4)", name, name))
                    }
                    Binding::Proc { ty: None, .. } => {
                        err(*line, format!("{} is a procedure without a type, and has no value to assign", name))
                    }
                    Binding::Slot(_, Kind::Array) | Binding::Own(_, Kind::Array) => {
                        err(*line, format!("{} is an array; a subscript is needed", name))
                    }
                    Binding::Label(_) => err(*line, format!("{} is a label", name)),
                    Binding::Switch(_) => err(*line, format!("{} is a switch", name)),
                    Binding::Builtin(_) => err(*line, format!("{} is a standard procedure", name)),
                }
            }
            E::Sub(name, subs, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Slot(_, Kind::Array | Kind::Formal(_)) | Binding::Own(_, Kind::Array) => {
                        let var = Self::var(name, hops, &b);
                        let mut v = Vec::new();
                        for s in subs {
                            v.push(self.expr(s, false)?);
                        }
                        Ok(Target::Index(var, v))
                    }
                    _ => err(*line, format!("{} is not an array", name)),
                }
            }
            _ => unreachable!(),
        }
    }

    fn actuals(&mut self, args: &[E]) -> CResult<Vec<Arg>> {
        let mut v = Vec::new();
        for a in args {
            if let (true, E::Id(name, line)) = (self.x1(), a) {
                if let Ok((_, Binding::Builtin(bi))) = self.lookup(name, *line) {
                    if bi.x1_operator() {
                        // Kruseman Aretz 2003, chapter 7; the compiler did
                        // not refuse it, but could produce bad code (see
                        // the notes of sergev/x1-algol-compiler).
                        self.warnings.push(format!(
                            "line {}: {} was an operator of the X1's run-time complex and could not be an actual parameter (Kruseman Aretz 2003, chapter 7)",
                            line, name
                        ));
                    }
                }
            }
            v.push(Rc::new(self.expr(a, true)?));
        }
        Ok(v)
    }

    fn subs(&mut self, subs: &[E]) -> CResult<Vec<Ex>> {
        subs.iter().map(|s| self.expr(s, false)).collect()
    }

    /// `actual` is true for an actual parameter, which may also be a
    /// string, an array, a procedure, a switch or a label.
    fn expr(&mut self, e: &E, actual: bool) -> CResult<Ex> {
        Ok(match e {
            // On the X1 a number without a point or exponent is real if it
            // is larger than the largest integer, and zero is always an
            // integer, even written 0.0 (property 4).
            E::Int(n) if self.x1() && *n > X1_MAXINT => Ex::Const(Value::Real(*n as f64)),
            E::Real(x) if self.x1() && *x == 0.0 => Ex::Const(Value::Int(0)),
            // A real constant is two words of the constant list, the format
            // of a real variable (x1algol.pas, look_for_constant), so its
            // mantissa has 40 bits.
            E::Real(x) if self.x1() => Ex::Const(Value::Real(crate::value::round_mantissa(*x, 40))),
            E::Int(n) => Ex::Const(Value::Int(*n)),
            E::Real(x) => Ex::Const(Value::Real(*x)),
            E::Bool(b) => Ex::Const(Value::Bool(*b)),
            E::Str(s) => {
                if !actual {
                    return Err("a string can only be an actual parameter (3.0, 4.7.1)".into());
                }
                Ex::Const(Value::Str(s.as_str().into()))
            }
            E::Id(name, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Label(id) => {
                        if !actual {
                            return err(*line, format!("{} is a label, not a value", name));
                        }
                        Ex::Label { hops, id }
                    }
                    Binding::Slot(_, Kind::Array) | Binding::Own(_, Kind::Array) if !actual => {
                        return err(*line, format!("{} is an array; a subscript is needed", name))
                    }
                    Binding::Switch(_) if !actual => return err(*line, format!("{} is a switch, not a value", name)),
                    Binding::Proc { ty, nparams, .. } if !actual => {
                        if ty.is_none() {
                            return err(*line, format!("{} is a procedure without a type, so it has no value", name));
                        }
                        if nparams != 0 {
                            return err(*line, format!("{} has {} parameters, but is given none", name, nparams));
                        }
                        Ex::Load(Self::var(name, hops, &b))
                    }
                    Binding::Builtin(bi) if !actual => {
                        if !bi.has_value() {
                            return err(*line, format!("{} has no value", name));
                        }
                        if bi.arity() != 0 {
                            return err(*line, format!("{} has {} parameters, but is given none", name, bi.arity()));
                        }
                        Ex::Load(Self::var(name, hops, &b))
                    }
                    _ => Ex::Load(Self::var(name, hops, &b)),
                }
            }
            E::Sub(name, subs, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Slot(_, Kind::Array | Kind::Formal(_)) | Binding::Own(_, Kind::Array) => {
                        Ex::Index(Self::var(name, hops, &b), self.subs(subs)?)
                    }
                    Binding::Switch(_) if actual => Ex::Index(Self::var(name, hops, &b), self.subs(subs)?),
                    Binding::Switch(_) => return err(*line, format!("{} is a switch, not an array", name)),
                    _ => return err(*line, format!("{} is not an array", name)),
                }
            }
            E::Call(name, args, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Proc { ty: None, .. } => {
                        return err(*line, format!("{} is a procedure without a type, so it has no value", name))
                    }
                    Binding::Proc { nparams, .. } if nparams != args.len() => {
                        return err(*line, format!("{} has {} parameters, but is given {}", name, nparams, args.len()))
                    }
                    Binding::Builtin(bi) if !bi.has_value() => return err(*line, format!("{} has no value", name)),
                    Binding::Builtin(bi) if bi.arity() != args.len() => {
                        return err(*line, format!("{} has {} parameters, but is given {}", name, bi.arity(), args.len()))
                    }
                    Binding::Proc { .. } | Binding::Builtin(_) | Binding::Slot(_, Kind::Formal(_)) => {}
                    _ => return err(*line, format!("{} is not a procedure", name)),
                }
                Ex::Call(Self::var(name, hops, &b), self.actuals(args)?)
            }
            E::Neg(a) => Ex::Neg(Box::new(self.expr(a, false)?)),
            E::Pos(a) => Ex::Pos(Box::new(self.expr(a, false)?)),
            E::Not(a) => Ex::Not(Box::new(self.expr(a, false)?)),
            E::Bin(op, a, b) => Ex::Bin(*op, Box::new(self.expr(a, false)?), Box::new(self.expr(b, false)?)),
            E::If(c, a, b) => {
                Ex::If(Box::new(self.expr(c, false)?), Box::new(self.expr(a, actual)?), Box::new(self.expr(b, actual)?))
            }
        })
    }

    /// A designational expression (3.5): a label, a switch designator, or a
    /// conditional one. Here an unsigned integer is a label.
    fn desig(&mut self, e: &E) -> CResult<Ex> {
        Ok(match e {
            E::Int(n) => {
                let name = n.to_string();
                match self.lookup(&name, 0) {
                    Ok((hops, Binding::Label(id))) => Ex::Label { hops, id },
                    _ => return Err(format!("there is no label {} here", n)),
                }
            }
            E::Id(name, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Label(id) => Ex::Label { hops, id },
                    Binding::Slot(_, Kind::Formal(_)) => Ex::Load(Self::var(name, hops, &b)),
                    Binding::Switch(_) => return err(*line, format!("{} is a switch; a subscript is needed", name)),
                    _ => return err(*line, format!("{} is not a label", name)),
                }
            }
            E::Sub(name, subs, line) => {
                let (hops, b) = self.lookup(name, *line)?;
                match b {
                    Binding::Switch(_) | Binding::Slot(_, Kind::Formal(_)) => {
                        Ex::Index(Self::var(name, hops, &b), self.subs(subs)?)
                    }
                    _ => return err(*line, format!("{} is not a switch", name)),
                }
            }
            E::If(c, a, b) => Ex::If(Box::new(self.expr(c, false)?), Box::new(self.desig(a)?), Box::new(self.desig(b)?)),
            _ => return Err("a designational expression (a label or a switch designator) expected".into()),
        })
    }
}
