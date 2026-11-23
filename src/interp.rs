//! The interpreter.
//!
//! Call by name is done as the copy rule of 4.7.3.2 requires in effect: an
//! actual parameter called by name becomes a thunk, its expression and the
//! frame of the call, and every use of the formal evaluates it again there.
//! Nothing is evaluated once and kept, nothing is evaluated that the Report
//! does not evaluate, and the order is the Report's: in an assignment the
//! subscripts of the left part first, then the expression (4.2.3); in a for
//! statement the expansion of 4.6.4.2 exactly, so the step and the limit
//! are evaluated afresh every time round.

use crate::ast::{BOp, Spec, Type};
use crate::compile::{Dialect, X1_MAXINT};
use crate::builtins::Builtin;

// The reals of the X1 (X1–X8 comparison, 3.2). A variable holds a 40-bit
// mantissa and a 12-bit exponent, so its magnitude lies between 2^−2048
// (about .3094⏨−616) and about .1616⏨+617; "the value 0 being replaced by
// about 10^−600". On the stack the exponent has 27 bits, and there 0 is
// "the smallest positive real number that can be represented as anonymous
// number in the accumulator" (X1 user manual, in appendix B.1 of the same
// report). f64 reaches neither, so three stand-ins are used, in the same
// order of size: X1_REAL_MIN for the least variable, x1_zero() below it for
// the stack's zero, and x1_real_max() for the greatest variable. FLOT and
// print show the first and last with the X1's own digits.
const X1_REAL_MIN: f64 = f64::MIN_POSITIVE;

fn x1_zero() -> f64 {
    f64::MIN_POSITIVE / 2f64.powi(40)
}

fn x1_real_max() -> f64 {
    (2f64.powi(40) - 1.0) * 2f64.powi(1024 - 40)
}

/// A real stored in a variable of the X1: "rounded to 12 decimals" (40
/// bits), and if "too large or too close to 0", the greatest or least
/// magnitude "with the sign of the result to be assigned" (X1 user manual,
/// complex ALD).
fn x1_store(x: f64) -> f64 {
    if x.abs() < X1_REAL_MIN {
        return if x < 0.0 { -X1_REAL_MIN } else { X1_REAL_MIN };
    }
    let r = round_mantissa(x, 40);
    if !r.is_finite() || r.abs() > x1_real_max() {
        x.signum() * x1_real_max()
    } else {
        r
    }
}
use crate::compile::*;
use crate::value::*;
use std::io::{BufRead, Write};
use std::rc::Rc;

pub enum Signal {
    Goto(LabelVal),
    Error(String),
    Stop,
}

type R<T> = Result<T, Signal>;

enum Loc {
    Slot(Rc<Frame>, usize, Option<Type>),
    Own(usize, Type),
    Elem(Rc<Array>, usize),
}

/// How much of the stack the interpreter may use before it stops a program
/// with an error rather than overflow. The thread is started with more
/// than this (see lib.rs). On WebAssembly there are no threads, and the
/// stack is the 64 MB the linker is asked for in .cargo/config.toml.
#[cfg(not(target_family = "wasm"))]
pub const STACK_BUDGET: usize = 3 << 30;
#[cfg(target_family = "wasm")]
pub const STACK_BUDGET: usize = 56 << 20;

fn stack_address() -> usize {
    let x = 0u8;
    std::hint::black_box(&x) as *const u8 as usize
}

pub struct Interp<'a> {
    prog: &'a Program,
    owns: Vec<Value>,
    out: &'a mut (dyn Write + Send),
    input: &'a mut (dyn BufRead + Send),
    inbuf: Vec<char>,
    inpos: usize,
    /// The position on the line, for TAB and the line width of 144.
    col: usize,
    steps: u64,
    max_steps: u64,
    next_id: u64,
    stack_base: usize,
    line: u32,
}

fn up(mut f: &Rc<Frame>, hops: u16) -> &Rc<Frame> {
    for _ in 0..hops {
        f = f.parent.as_ref().unwrap();
    }
    f
}

impl<'a> Interp<'a> {
    pub fn new(
        prog: &'a Program,
        out: &'a mut (dyn Write + Send),
        input: &'a mut (dyn BufRead + Send),
        max_steps: Option<u64>,
    ) -> Self {
        Interp {
            prog,
            owns: prog.owns.clone(),
            out,
            input,
            inbuf: Vec::new(),
            inpos: 0,
            col: 0,
            steps: 0,
            max_steps: max_steps.unwrap_or(u64::MAX),
            next_id: 0,
            stack_base: stack_address(),
            line: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        let root = self.frame(None, 0);
        let prog = self.prog;
        let r = self.block(&prog.main, &root);
        let _ = self.out.flush();
        match r {
            Ok(()) | Err(Signal::Stop) => Ok(()),
            Err(Signal::Error(e)) => Err(e),
            Err(Signal::Goto(l)) => Err(format!(
                "go to {}: that label cannot be reached from here (a jump into a for statement?)",
                prog.labels[l.id as usize]
            )),
        }
    }

    fn x1(&self) -> bool {
        self.prog.dialect == Dialect::X1
    }

    /// A run-time stop of the X1 system, with its number from the user
    /// manual of 1962 (Kruseman Aretz 2003, appendix A).
    fn x1_stop<T>(&self, number: &str, msg: &str) -> R<T> {
        self.fail(format!("X1 stop {}: {}", number, msg))
    }

    /// An integer result. On the X1 one beyond ±67108863 "is transformed
    /// automatically to real representation" (X1 user manual); the stop
    /// 1–1 comes only when such a value is assigned to an integer.
    fn int(&self, n: i64) -> R<Value> {
        if self.x1() && n.abs() > X1_MAXINT {
            return Ok(Value::Real(n as f64));
        }
        Ok(Value::Int(n))
    }

    /// A value as stored in a variable: on the X1 a real is rounded to 40
    /// bits and kept within the range of variables.
    fn stored(&self, v: Value) -> Value {
        match v {
            Value::Real(x) if self.x1() => Value::Real(x1_store(x)),
            v => v,
        }
    }

    #[cold]
    #[inline(never)]
    fn fail<T>(&self, msg: impl AsRef<str>) -> R<T> {
        Err(Signal::Error(format!("line {}: {}", self.line, msg.as_ref())))
    }

    fn frame(&mut self, parent: Option<Rc<Frame>>, n: usize) -> Rc<Frame> {
        self.next_id += 1;
        Rc::new(Frame { id: self.next_id, parent, slots: std::cell::RefCell::new(vec![Value::Undef; n]) })
    }

    fn check_stack(&self) -> R<()> {
        if self.stack_base.abs_diff(stack_address()) > STACK_BUDGET {
            return self.fail(format!(
                "procedures nested too deep: the {} MB of stack is used up",
                STACK_BUDGET >> 20
            ));
        }
        Ok(())
    }

    fn tick(&mut self) -> R<()> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return self.fail(format!("stopped after {} steps", self.max_steps));
        }
        Ok(())
    }

    // ----- statements -----

    #[inline(always)]
    fn block(&mut self, b: &'a BlockCode, env: &Rc<Frame>) -> R<()> {
        let f = self.frame(Some(env.clone()), b.nslots);
        if !b.inits.is_empty() {
            self.declare(b, &f)?;
        }
        self.exec_code(&b.body, &f)
    }

    /// The declarations of a block: simple variables and arrays. Kept out
    /// of block() so that the frames of a deep recursion stay small (the
    /// browser's stack is about 1 MB).
    #[inline(never)]
    fn declare(&mut self, b: &'a BlockCode, f: &Rc<Frame>) -> R<()> {
        for init in &b.inits {
            match init {
                Init::Simple { slot, ty } => f.slots.borrow_mut()[*slot as usize] = Value::zero(*ty),
                Init::Array { place, ty, bounds, name } => {
                    if let ArrPlace::Own(i) = place {
                        if !matches!(self.owns[*i as usize], Value::Undef) {
                            continue;
                        }
                    }
                    let mut bs = Vec::new();
                    let mut size: usize = 1;
                    for (lo, hi) in bounds.iter() {
                        let l = self.eval(lo, f)?;
                        let l = self.to_int(l)?;
                        let h = self.eval(hi, f)?;
                        let h = self.to_int(h)?;
                        if h < l && self.x1() {
                            return self.x1_stop(
                                "1–2",
                                &format!("in the declaration of {} a lower bound is larger than the upper bound", name),
                            );
                        }
                        let n = if h < l { 0 } else { (h - l + 1) as usize };
                        size = match size.checked_mul(n).filter(|&s| s <= 100_000_000) {
                            Some(s) => s,
                            None => return self.fail(format!("array {} is too large", name)),
                        };
                        bs.push((l, h));
                    }
                    let a = Rc::new(Array { ty: *ty, bounds: bs, data: std::cell::RefCell::new(vec![Value::zero(*ty); size]) });
                    match place {
                        ArrPlace::Slot(s) => f.slots.borrow_mut()[*s as usize] = Value::Array(a),
                        ArrPlace::Own(i) => self.owns[*i as usize] = Value::Array(a),
                    }
                }
            }
        }
        Ok(())
    }

    fn exec_code(&mut self, code: &'a Code, env: &Rc<Frame>) -> R<()> {
        let mut pc = 0;
        while pc < code.ins.len() {
            let (ins, line) = &code.ins[pc];
            self.line = *line;
            let r = self.tick().and_then(|_| self.exec(ins, env));
            match r {
                Ok(None) => pc += 1,
                Ok(Some(next)) => pc = next,
                Err(Signal::Goto(l)) if l.frame == env.id => match code.find(l.id) {
                    Some(p) => pc = p,
                    None => return Err(Signal::Goto(l)),
                },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// A left part list of more than one variable (4.2.3).
    #[inline(always)]
    fn assign_many(&mut self, targets: &[Target], e: &Ex, env: &Rc<Frame>) -> R<()> {
        let mut locs = Vec::with_capacity(targets.len());
        for t in targets {
            locs.push(self.locate(t, env)?);
        }
        let v = self.eval(e, env)?;
        for loc in &locs {
            self.store(loc, v.clone())?;
        }
        Ok(())
    }

    #[inline(never)]
    fn goto(&mut self, e: &Ex, env: &Rc<Frame>) -> R<Option<usize>> {
        // A switch designator whose value is undefined makes the go
        // to statement a dummy statement (4.3.5).
        match self.desig(e, env)? {
            Some(l) => return Err(Signal::Goto(l)),
            // On the X1 it is undefined instead (property 10).
            None if self.x1() => {
                return self.fail(
                    "go to an undefined switch designator: a dummy statement by the Report (4.3.5), undefined on the X1 (property 10)",
                )
            }
            None => {}
        }
        Ok(None)
    }

    /// Returns the next instruction, if it is not the following one.
    #[inline(always)]
    fn exec(&mut self, ins: &'a Ins, env: &Rc<Frame>) -> R<Option<usize>> {
        match ins {
            Ins::Assign(targets, e) => {
                if let [t] = &targets[..] {
                    let loc = self.locate(t, env)?;
                    let v = self.eval(e, env)?;
                    self.store(&loc, v)?;
                } else {
                    self.assign_many(targets, e, env)?;
                }
            }
            Ins::Call(var, args) => {
                self.call_var(var, args, env, false)?;
            }
            Ins::Goto(e) => return self.goto(e, env),
            Ins::JumpUnless(c, to) => {
                if !self.eval_bool(c, env)? {
                    return Ok(Some(*to));
                }
            }
            Ins::Jump(to) => return Ok(Some(*to)),
            Ins::Block(b) => self.block(b, env)?,
            Ins::For(f) => self.for_statement(f, env)?,
        }
        Ok(None)
    }

    /// 4.6.4: the for list elements, expanded as the Report expands them.
    #[inline(never)]
    fn for_statement(&mut self, f: &'a ForCode, env: &Rc<Frame>) -> R<()> {
        for el in &f.elems {
            match el {
                ForEl::Expr(e) => {
                    let loc = self.locate(&f.var, env)?;
                    let v = self.eval(e, env)?;
                    self.store(&loc, v)?;
                    self.exec_code(&f.body, env)?;
                }
                ForEl::Step(a, b, c) if self.x1() => self.x1_step(f, a, b, c, env)?,
                ForEl::Step(a, b, c) => {
                    // V := A;
                    let loc = self.locate(&f.var, env)?;
                    let v = self.eval(a, env)?;
                    self.store(&loc, v)?;
                    loop {
                        // L1: if (V - C) × sign(B) > 0 then go to exhausted;
                        self.tick()?;
                        let v = self.eval(&f.var_ex, env)?;
                        let cv = self.eval(c, env)?;
                        let bv = self.eval(b, env)?;
                        let d = self.arith(BOp::Sub, v, cv)?;
                        let s = self.sign(&bv)?;
                        let p = self.arith(BOp::Mul, d, Value::Int(s))?;
                        let done = match p {
                            Value::Int(n) => n > 0,
                            Value::Real(x) => x > 0.0,
                            _ => unreachable!(),
                        };
                        if done {
                            break;
                        }
                        // S;
                        self.exec_code(&f.body, env)?;
                        // V := V + B; go to L1
                        let loc = self.locate(&f.var, env)?;
                        let v = self.eval(&f.var_ex, env)?;
                        let bv = self.eval(b, env)?;
                        let nv = self.arith(BOp::Add, v, bv)?;
                        self.store(&loc, nv)?;
                    }
                }
                ForEl::While(e, cond) => loop {
                    // L3: V := E; if ¬F then go to exhausted; S; go to L3
                    self.tick()?;
                    let loc = self.locate(&f.var, env)?;
                    let v = self.eval(e, env)?;
                    self.store(&loc, v)?;
                    if !self.eval_bool(cond, env)? {
                        break;
                    }
                    self.exec_code(&f.body, env)?;
                },
            }
        }
        Ok(())
    }

    /// The step-until element as the X1's run-time routines FOR1 and FOR5
    /// to FOR7 did it (Kruseman Aretz, X1–X8 comparison, 3.3.3): the step
    /// is evaluated once each time round, and the first time before the
    /// initial value is assigned (property 11 of the 1962 manual).
    ///
    ///   first:  address of V; A; B; V := A; C; test
    ///   later:  address of V; value of V; B; V := V + B; C; test
    #[inline(never)]
    fn x1_step(&mut self, f: &'a ForCode, a: &Ex, b: &Ex, c: &Ex, env: &Rc<Frame>) -> R<()> {
        let mut loc = self.locate(&f.var, env)?;
        let av = self.eval(a, env)?;
        let mut bv = self.eval(b, env)?;
        self.store(&loc, av)?;
        loop {
            self.tick()?;
            let cv = self.eval(c, env)?;
            let v = self.load_loc(&loc);
            let d = self.arith(BOp::Sub, v, cv)?;
            let s = self.sign(&bv)?;
            let p = self.arith(BOp::Mul, d, Value::Int(s))?;
            if match p {
                Value::Int(n) => n > 0,
                Value::Real(x) => x > 0.0,
                _ => unreachable!(),
            } {
                return Ok(());
            }
            self.exec_code(&f.body, env)?;
            loc = self.locate(&f.var, env)?;
            let v = self.load_loc(&loc);
            bv = self.eval(b, env)?;
            let nv = self.arith(BOp::Add, v, bv.clone())?;
            self.store(&loc, nv)?;
        }
    }

    fn load_loc(&self, loc: &Loc) -> Value {
        match loc {
            Loc::Slot(f, s, _) => f.slots.borrow()[*s].clone(),
            Loc::Own(i, _) => self.owns[*i].clone(),
            Loc::Elem(a, i) => a.data.borrow()[*i].clone(),
        }
    }

    // ----- procedures -----

    fn get(&self, var: &Var, env: &Rc<Frame>) -> Value {
        match var.what {
            What::Slot(s, _) => up(env, var.hops).slots.borrow()[s as usize].clone(),
            What::Own(i, _) => self.owns[i as usize].clone(),
            What::Proc(id) => Value::Proc(Rc::new(Closure { id, env: up(env, var.hops).clone() })),
            What::Switch(id) => Value::Switch(Rc::new(SwitchVal { id, env: up(env, var.hops).clone() })),
            What::Builtin(b) => Value::Builtin(b),
        }
    }

    /// What an identifier stands for, following formal parameters called
    /// by name back to their actual parameters, but not evaluating them.
    fn resolve(&mut self, var: &Var, env: &Rc<Frame>) -> R<Value> {
        match self.get(var, env) {
            Value::Name(t) => self.resolve_ex(&t.ex, &t.env),
            v => Ok(v),
        }
    }

    fn resolve_ex(&mut self, ex: &Ex, env: &Rc<Frame>) -> R<Value> {
        match ex {
            Ex::Load(var) => self.resolve(var, env),
            _ => self.eval(ex, env),
        }
    }

    #[inline(always)]
    fn call_var(&mut self, var: &Var, args: &[Arg], env: &Rc<Frame>, want_value: bool) -> R<Value> {
        match var.what {
            What::Proc(id) => {
                let fenv = up(env, var.hops).clone();
                self.call_proc(id, fenv, args, env, want_value)
            }
            What::Builtin(b) => self.builtin(b, args, env),
            _ => match self.resolve(var, env)? {
                Value::Proc(c) => self.call_proc(c.id, c.env.clone(), args, env, want_value),
                Value::Builtin(b) => {
                    if b.arity() != args.len() {
                        return self.fail(format!("{} has {} parameters, but is given {}", b.name(), b.arity(), args.len()));
                    }
                    if want_value && !b.has_value() {
                        return self.fail(format!("{} has no value", b.name()));
                    }
                    self.builtin(b, args, env)
                }
                v => self.fail(format!("{} is {}, not a procedure", var.name, v.kind())),
            },
        }
    }

    fn call_proc(&mut self, id: u32, fenv: Rc<Frame>, args: &[Arg], caller: &Rc<Frame>, want_value: bool) -> R<Value> {
        let prog = self.prog;
        let code = &prog.procs[id as usize];
        if args.len() != code.params.len() {
            return self.fail(format!("{} has {} parameters, but is given {}", code.name, code.params.len(), args.len()));
        }
        if want_value && code.ty.is_none() {
            return self.fail(format!("{} is a procedure without a type, so it has no value", code.name));
        }
        self.tick()?;
        self.check_stack()?;
        let frame = self.frame(Some(fenv), code.nslots);
        frame.slots.borrow_mut()[0] = Value::zero(code.ty.unwrap_or(Type::Integer));
        for (i, (p, arg)) in code.params.iter().zip(args).enumerate() {
            if !p.by_value {
                let v = self.bind(p, arg, caller)?;
                frame.slots.borrow_mut()[i + 1] = v;
            }
        }
        if !code.value_order.is_empty() {
            self.bind_values(code, args, caller, &frame)?;
        }
        let saved = self.line;
        // A body that is one block is entered directly: one Rust frame
        // fewer for each call, which is what lets man or boy reach k = 10
        // on a browser's stack.
        let r = match &code.body.ins[..] {
            [(Ins::Block(b), line)] if code.body.labels.is_empty() => {
                self.line = *line;
                self.tick().and_then(|_| self.block(b, &frame))
            }
            _ => self.exec_code(&code.body, &frame),
        };
        self.line = saved;
        r?;
        let v = frame.slots.borrow()[0].clone();
        Ok(v)
    }

    /// The parameters called by value, in the order the Report's value
    /// assignment makes them (4.7.3.1). Out of line, to keep call_proc's
    /// frame small.
    #[inline(never)]
    fn bind_values(&mut self, code: &'a ProcCode, args: &[Arg], caller: &Rc<Frame>, frame: &Rc<Frame>) -> R<()> {
        for &i in &code.value_order {
            let v = self.bind(&code.params[i], &args[i], caller)?;
            // A parameter called by value is a local variable, so on the
            // X1 a real one is stored with a 40-bit mantissa.
            let v = match v {
                Value::Real(x) if self.x1() => Value::Real(x1_store(x)),
                Value::Array(a) if self.x1() && a.ty == Type::Real => {
                    for e in a.data.borrow_mut().iter_mut() {
                        if let Value::Real(x) = e {
                            *x = x1_store(*x);
                        }
                    }
                    Value::Array(a)
                }
                v => v,
            };
            if let (true, Value::Array(a)) = (self.x1(), &v) {
                if a.bounds.len() > 5 {
                    return self.fail(format!(
                        "the X1 system does not accept this (property 23 of its manual): the array {} in the value list has more than five indices",
                        code.params[i].name
                    ));
                }
            }
            frame.slots.borrow_mut()[i + 1] = v;
        }
        Ok(())
    }

    /// 4.7.3: the value assignment for a parameter called by value, and for
    /// one called by name the thunk standing for the actual parameter.
    #[inline(never)]
    fn bind(&mut self, p: &ParamCode, arg: &Arg, env: &Rc<Frame>) -> R<Value> {
        if p.by_value {
            return match p.spec {
                Some(Spec::Simple(t)) => {
                    let v = self.eval(arg, env)?;
                    self.convert(v, t)
                }
                Some(Spec::Array(t)) => match self.resolve_ex(arg, env)? {
                    Value::Array(a) => {
                        let mut data = Vec::with_capacity(a.data.borrow().len());
                        for v in a.data.borrow().iter() {
                            data.push(self.convert(v.clone(), t)?);
                        }
                        Ok(Value::Array(Rc::new(Array { ty: t, bounds: a.bounds.clone(), data: std::cell::RefCell::new(data) })))
                    }
                    v => self.fail(format!("the actual parameter for {} is {}, not an array", p.name, v.kind())),
                },
                Some(Spec::Label) => match self.desig(arg, env)? {
                    Some(l) => Ok(Value::Label(l)),
                    None => self.fail(format!("the actual parameter for {} is an undefined switch designator", p.name)),
                },
                _ => match self.eval(arg, env)? {
                    Value::Array(a) => Ok(Value::Array(Rc::new(Array {
                        ty: a.ty,
                        bounds: a.bounds.clone(),
                        data: std::cell::RefCell::new(a.data.borrow().clone()),
                    }))),
                    v => Ok(v),
                },
            };
        }
        match &**arg {
            Ex::Load(var) => match self.get(var, env) {
                v @ (Value::Name(_)
                | Value::Array(_)
                | Value::Proc(_)
                | Value::Builtin(_)
                | Value::Switch(_)
                | Value::Label(_)
                | Value::Str(_)) => Ok(v),
                _ => Ok(Value::Name(Rc::new(Thunk { ex: arg.clone(), env: env.clone() }))),
            },
            Ex::Label { .. } | Ex::Const(Value::Str(_)) => self.eval(arg, env),
            _ => Ok(Value::Name(Rc::new(Thunk { ex: arg.clone(), env: env.clone() }))),
        }
    }

    // ----- variables -----

    fn locate(&mut self, t: &Target, env: &Rc<Frame>) -> R<Loc> {
        match t {
            Target::Var(var) => self.locate_var(var, env),
            Target::Index(var, subs) => self.locate_index(var, subs, env),
            // The X1 did not convert the value of a function to its declared
            // type: "the type of the result is determined by the arithmetic
            // that is carried out this time" (property 18).
            Target::Result { hops, ty } => {
                Ok(Loc::Slot(up(env, *hops).clone(), 0, if self.x1() { None } else { Some(*ty) }))
            }
        }
    }

    fn locate_var(&mut self, var: &Var, env: &Rc<Frame>) -> R<Loc> {
        match var.what {
            What::Slot(s, Kind::Simple(t)) => Ok(Loc::Slot(up(env, var.hops).clone(), s as usize, Some(t))),
            What::Own(i, Kind::Simple(t)) => Ok(Loc::Own(i as usize, t)),
            What::Slot(s, Kind::Formal(conv)) => match self.get(var, env) {
                Value::Name(t) => match &*t.ex {
                    Ex::Load(v) => self.locate_var(v, &t.env),
                    Ex::Index(v, subs) => self.locate_index(v, subs, &t.env),
                    _ => self.fail(format!(
                        "{} is assigned to, but its actual parameter is an expression, not a variable",
                        var.name
                    )),
                },
                Value::Int(_) | Value::Real(_) | Value::Bool(_) | Value::Undef => {
                    Ok(Loc::Slot(up(env, var.hops).clone(), s as usize, conv))
                }
                v => self.fail(format!("{} is assigned to, but its actual parameter is {}", var.name, v.kind())),
            },
            What::Proc(_) => self.fail(format!(
                "{} is assigned to, but its actual parameter is a procedure, not a variable",
                var.name
            )),
            _ => self.fail(format!("{} is not a simple variable", var.name)),
        }
    }

    fn locate_index(&mut self, var: &Var, subs: &[Ex], env: &Rc<Frame>) -> R<Loc> {
        match self.resolve(var, env)? {
            Value::Array(a) => {
                let i = self.index(&a, subs, env, &var.name)?;
                Ok(Loc::Elem(a, i))
            }
            v => self.fail(format!("{} is {}, not an array", var.name, v.kind())),
        }
    }

    #[inline(never)]
    fn index(&mut self, a: &Array, subs: &[Ex], env: &Rc<Frame>, name: &str) -> R<usize> {
        if subs.len() != a.bounds.len() {
            return self.fail(format!("{} has {} subscripts, but is given {}", name, a.bounds.len(), subs.len()));
        }
        let mut off: usize = 0;
        for (s, &(lo, hi)) in subs.iter().zip(&a.bounds) {
            let v = self.eval(s, env)?;
            let i = self.to_int(v)?;
            if i < lo || i > hi {
                return self.fail(format!("subscript {} is outside the bounds [{}:{}] of {}", i, lo, hi, name));
            }
            off = off * (hi - lo + 1) as usize + (i - lo) as usize;
        }
        Ok(off)
    }

    #[inline(never)]
    fn store(&mut self, loc: &Loc, v: Value) -> R<()> {
        // A real variable of the X1 holds a mantissa of 40 bits, against 52
        // on the stack (X1–X8 comparison, 3.2), and the store rounds (X1
        // user manual: "rounded to 12 decimals"). The conversion comes
        // first, so that an integer 0 stored in a real is the X1's least
        // real, and a real stored in an integer is rounded from the stack.
        match loc {
            Loc::Slot(f, s, conv) => {
                let v = match conv {
                    Some(t) => self.convert(v, *t)?,
                    None => v,
                };
                f.slots.borrow_mut()[*s] = self.stored(v);
            }
            Loc::Own(i, t) => {
                let v = self.convert(v, *t)?;
                self.owns[*i] = self.stored(v);
            }
            Loc::Elem(a, i) => {
                let v = self.convert(v, a.ty)?;
                a.data.borrow_mut()[*i] = self.stored(v);
            }
        }
        Ok(())
    }

    /// Assignment to an integer rounds: entier(E + 0.5) (4.2.4).
    fn convert(&self, v: Value, t: Type) -> R<Value> {
        match (v, t) {
            (v @ Value::Int(_), Type::Integer) | (v @ Value::Real(_), Type::Real) | (v @ Value::Bool(_), Type::Boolean) => {
                Ok(v)
            }
            (Value::Int(i), Type::Real) => Ok(Value::Real(self.operand(&Value::Int(i))?)),
            (Value::Real(x), Type::Integer) => {
                let n = self.round(x)?;
                if self.x1() && n.abs() > X1_MAXINT {
                    return self.x1_stop("1–1", &format!("an integer value ({}) exceeds the integer capacity", n));
                }
                Ok(Value::Int(n))
            }
            (v, t) => self.fail(format!("{} cannot be assigned to a variable of type {}", v.kind(), t.name())),
        }
    }

    /// The Report's transfer function, entier(x + 0.5). The X1 rounds
    /// (MR 35; X1 user manual), and Dijkstra's course calls a half
    /// undefined ("x := 7/2 ... x can just as well become 3 as 4"). Here
    /// rounding is of the magnitude, so a half goes away from zero: the X1's arithmetic was ones' complement, in which a shift
    /// truncates towards zero; its compiler rounds constants by adding half
    /// a unit to the magnitude (AFRONDING in FT4, [KA03, ch. 10]); and the
    /// X8 rounded halves away from zero.
    fn round(&self, x: f64) -> R<i64> {
        let r = if self.x1() { x.round() } else { (x + 0.5).floor() };
        if !(-9.2e18..9.2e18).contains(&r) {
            return self.fail(format!("{} is too large for an integer", format_real(x)));
        }
        Ok(r as i64)
    }

    /// Subscripts and bounds act like integer variables (3.1.4.2).
    fn to_int(&self, v: Value) -> R<i64> {
        match v {
            Value::Int(i) => Ok(i),
            Value::Real(x) => self.round(x),
            v => self.fail(format!("an arithmetic value expected, but found {}", v.kind())),
        }
    }

    // ----- expressions -----

    fn load(&mut self, var: &Var, env: &Rc<Frame>) -> R<Value> {
        match var.what {
            What::Proc(id) => {
                let fenv = up(env, var.hops).clone();
                self.call_proc(id, fenv, &[], env, true)
            }
            What::Builtin(b) => self.builtin(b, &[], env),
            _ => match self.get(var, env) {
                Value::Name(t) => {
                    self.check_stack()?;
                    self.eval(&t.ex, &t.env)
                }
                Value::Proc(c) => self.call_proc(c.id, c.env.clone(), &[], env, true),
                Value::Builtin(b) => {
                    if b.arity() != 0 || !b.has_value() {
                        return self.fail(format!("{} cannot be used as a value", b.name()));
                    }
                    self.builtin(b, &[], env)
                }
                v => Ok(v),
            },
        }
    }

    pub fn eval(&mut self, ex: &Ex, env: &Rc<Frame>) -> R<Value> {
        match ex {
            Ex::Const(v) => Ok(v.clone()),
            Ex::Load(var) => self.load(var, env),
            Ex::Label { hops, id } => Ok(Value::Label(LabelVal { frame: up(env, *hops).id, id: *id })),
            Ex::Index(var, subs) => self.eval_index(var, subs, env),
            Ex::Call(var, args) => self.call_var(var, args, env, true),
            Ex::Neg(a) | Ex::Pos(a) => self.unary(ex, a, env),
            Ex::Not(a) => {
                let b = self.eval_bool(a, env)?;
                Ok(Value::Bool(!b))
            }
            Ex::Bin(op, a, b) => {
                // Both operands are always evaluated, left to right: the
                // Report has no short-circuit operators.
                let x = self.eval(a, env)?;
                let y = self.eval(b, env)?;
                self.binop(*op, x, y)
            }
            Ex::If(c, a, b) => {
                if self.eval_bool(c, env)? {
                    self.eval(a, env)
                } else {
                    self.eval(b, env)
                }
            }
        }
    }

    #[inline(never)]
    fn eval_index(&mut self, var: &Var, subs: &[Ex], env: &Rc<Frame>) -> R<Value> {
        match self.resolve(var, env)? {
        Value::Array(a) => {
            let i = self.index(&a, subs, env, &var.name)?;
            let v = a.data.borrow()[i].clone();
            Ok(v)
        }
        Value::Switch(s) => match self.switch_select(&s, subs, env, &var.name)? {
            Some(l) => Ok(Value::Label(l)),
            None => self.fail(format!("the switch designator {}[...] is undefined", var.name)),
        },
        v => self.fail(format!("{} is {}, not an array", var.name, v.kind())),
    }
    }

    #[inline(never)]
    fn unary(&mut self, ex: &Ex, a: &Ex, env: &Rc<Frame>) -> R<Value> {
        let v = self.eval(a, env)?;
        match (ex, v) {
            (Ex::Neg(_), Value::Int(i)) => match i.checked_neg() {
                Some(n) => Ok(Value::Int(n)),
                None => self.fail("integer overflow"),
            },
            (Ex::Neg(_), Value::Real(x)) => Ok(Value::Real(-x)),
            (Ex::Neg(_), v) => self.fail(format!("'-' needs an arithmetic operand, not {}", v.kind())),
            (_, v @ (Value::Int(_) | Value::Real(_))) => Ok(v),
            (_, v) => self.fail(format!("'+' needs an arithmetic operand, not {}", v.kind())),
        }
    }

    fn eval_bool(&mut self, ex: &Ex, env: &Rc<Frame>) -> R<bool> {
        match self.eval(ex, env)? {
            Value::Bool(b) => Ok(b),
            v => self.fail(format!("a Boolean value expected, but found {}", v.kind())),
        }
    }

    #[inline(never)]
    fn switch_select(&mut self, s: &SwitchVal, subs: &[Ex], env: &Rc<Frame>, name: &str) -> R<Option<LabelVal>> {
        if subs.len() != 1 {
            return self.fail(format!("the switch {} takes one subscript", name));
        }
        let v = self.eval(&subs[0], env)?;
        let i = self.to_int(v)?;
        let prog = self.prog;
        let elems = &prog.switches[s.id as usize];
        if i < 1 || i as usize > elems.len() {
            return Ok(None);
        }
        let senv = s.env.clone();
        self.desig(&elems[i as usize - 1], &senv)
    }

    /// The label a designational expression stands for, or None for an
    /// undefined switch designator.
    #[inline(never)]
    fn desig(&mut self, ex: &Ex, env: &Rc<Frame>) -> R<Option<LabelVal>> {
        match ex {
            Ex::Label { hops, id } => Ok(Some(LabelVal { frame: up(env, *hops).id, id: *id })),
            Ex::If(c, a, b) => {
                if self.eval_bool(c, env)? {
                    self.desig(a, env)
                } else {
                    self.desig(b, env)
                }
            }
            Ex::Index(var, subs) => match self.resolve(var, env)? {
                Value::Switch(s) => self.switch_select(&s, subs, env, &var.name),
                v => self.fail(format!("{} is {}, not a switch", var.name, v.kind())),
            },
            Ex::Load(var) => match self.get(var, env) {
                Value::Name(t) => self.desig(&t.ex, &t.env),
                Value::Label(l) => Ok(Some(l)),
                v => self.fail(format!("{} is {}, not a label", var.name, v.kind())),
            },
            _ => match self.eval(ex, env)? {
                Value::Label(l) => Ok(Some(l)),
                v => self.fail(format!("a label expected, but found {}", v.kind())),
            },
        }
    }

    // ----- arithmetic (3.3.4) -----

    fn num(&self, v: &Value) -> R<f64> {
        match v {
            Value::Int(i) => Ok(*i as f64),
            Value::Real(x) => Ok(*x),
            v => self.fail(format!("an arithmetic value expected, but found {}", v.kind())),
        }
    }

    /// An operand of an arithmetic operation or a relation. On the X1 the
    /// integer 0 becomes, as a real, the least positive number of the stack
    /// ("the transformation from integer to real is exact except for
    /// integer 0", X1 user manual).
    fn operand(&self, v: &Value) -> R<f64> {
        match v {
            Value::Int(0) if self.x1() => Ok(x1_zero()),
            v => self.num(v),
        }
    }

    /// A real result. On the X1 a result that would be exactly 0 is the
    /// least positive number of the stack, as for "sum or difference of two
    /// real numbers" in the X1 user manual. A result smaller still (0 / 2)
    /// is kept at that magnitude, since the stack has nothing smaller
    /// (assumed: the manual says only that results are "unambiguous"). And
    /// since the stack's exponent has 27 bits, a result beyond f64 is taken
    /// to the greatest real, which is what storing it would give.
    fn real(&self, x: f64) -> R<Value> {
        if self.x1() && x.abs() <= x1_zero() {
            return Ok(Value::Real(if x < 0.0 { -x1_zero() } else { x1_zero() }));
        }
        if self.x1() && x.is_infinite() {
            return Ok(Value::Real(x.signum() * x1_real_max()));
        }
        if x.is_finite() {
            Ok(Value::Real(x))
        } else {
            self.fail("real overflow")
        }
    }

    fn sign(&self, v: &Value) -> R<i64> {
        let x = self.num(v)?;
        Ok(if x > 0.0 {
            1
        } else if x < 0.0 {
            -1
        } else {
            0
        })
    }

    fn arith(&self, op: BOp, x: Value, y: Value) -> R<Value> {
        if let (Value::Int(a), Value::Int(b)) = (&x, &y) {
            let r = match op {
                BOp::Add => a.checked_add(*b),
                BOp::Sub => a.checked_sub(*b),
                BOp::Mul => a.checked_mul(*b),
                _ => unreachable!(),
            };
            return match r {
                Some(n) => self.int(n),
                None => self.fail("integer overflow"),
            };
        }
        let (a, b) = (self.operand(&x)?, self.operand(&y)?);
        self.real(match op {
            BOp::Add => a + b,
            BOp::Sub => a - b,
            BOp::Mul => a * b,
            _ => unreachable!(),
        })
    }

    #[inline(never)]
    fn binop(&self, op: BOp, x: Value, y: Value) -> R<Value> {
        match op {
            BOp::Add | BOp::Sub | BOp::Mul => self.arith(op, x, y),
            BOp::Div => {
                let (a, b) = (self.operand(&x)?, self.operand(&y)?);
                if b == 0.0 {
                    return self.fail("division by zero");
                }
                self.real(a / b)
            }
            BOp::IntDiv => match (x, y) {
                (Value::Int(_), Value::Int(0)) => self.fail("division by zero"),
                (Value::Int(a), Value::Int(b)) => match a.checked_div(b) {
                    Some(n) => self.int(n),
                    None => self.fail("integer overflow"),
                },
                (x, y) if self.x1() => self.x1_stop("1–8", &format!("the operands of ÷ are not both of type integer ({} and {})", x.kind(), y.kind())),
                (x, y) => self.fail(format!("'÷' is defined only for integers, but is given {} and {} (3.3.4.2)", x.kind(), y.kind())),
            },
            BOp::Pow => self.power(x, y),
            BOp::Lt | BOp::Le | BOp::Eq | BOp::Ge | BOp::Gt | BOp::Ne => {
                let ord = match (&x, &y) {
                    (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
                    (Value::Bool(_), _) | (_, Value::Bool(_)) => {
                        return self.fail("relations compare arithmetic values; for Boolean values use ≡ (3.4.1)")
                    }
                    _ => self.operand(&x)?.partial_cmp(&self.operand(&y)?),
                }
                .unwrap();
                use std::cmp::Ordering::*;
                Ok(Value::Bool(match op {
                    BOp::Lt => ord == Less,
                    BOp::Le => ord != Greater,
                    BOp::Eq => ord == Equal,
                    BOp::Ge => ord != Less,
                    BOp::Gt => ord == Greater,
                    _ => ord != Equal,
                }))
            }
            BOp::And | BOp::Or | BOp::Impl | BOp::Eqv => match (x, y) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(match op {
                    BOp::And => a && b,
                    BOp::Or => a || b,
                    BOp::Impl => !a || b,
                    _ => a == b,
                })),
                (x, y) => self.fail(format!("a Boolean operator is given {} and {}", x.kind(), y.kind())),
            },
        }
    }

    /// 3.3.4.3. The type of i↑j depends on the value of j: an integer for
    /// j ≥ 0, a real for j < 0.
    #[inline(never)]
    fn power(&self, x: Value, y: Value) -> R<Value> {
        match (&x, &y) {
            (Value::Int(i), Value::Int(j)) => {
                let (i, j) = (*i, *j);
                if j > 0 {
                    if i == 0 || i == 1 {
                        return Ok(Value::Int(i));
                    }
                    if i == -1 {
                        return Ok(Value::Int(if j % 2 == 0 { 1 } else { -1 }));
                    }
                    match u32::try_from(j).ok().and_then(|j| i.checked_pow(j)) {
                        Some(r) => self.int(r),
                        None if self.x1() => self.real((i as f64).powi(j as i32)),
                        None => self.fail("integer overflow in ↑"),
                    }
                } else if i == 0 {
                    self.fail(format!("0↑{} is undefined (3.3.4.3)", j))
                } else if j == 0 {
                    Ok(Value::Int(1))
                } else {
                    self.real(1.0 / (i as f64).powi(-j as i32))
                }
            }
            (Value::Real(r), Value::Int(j)) => {
                let (r, j) = (*r, *j);
                if j > 0 {
                    self.real(r.powi(j as i32))
                } else if r == 0.0 {
                    self.fail(format!("0.0↑{} is undefined (3.3.4.3)", j))
                } else if j == 0 {
                    Ok(Value::Real(1.0))
                } else {
                    self.real(1.0 / r.powi(-j as i32))
                }
            }
            (_, Value::Real(r)) => {
                let a = self.num(&x)?;
                let r = *r;
                if a > 0.0 {
                    self.real((r * a.ln()).exp())
                } else if a == 0.0 && r > 0.0 {
                    Ok(Value::Real(0.0))
                } else if a == 0.0 {
                    self.fail(format!("0↑{} is undefined (3.3.4.3)", format_real(r)))
                } else {
                    self.fail(format!("{}↑{} is undefined: a negative number to a real power (3.3.4.3)", format_real(a), format_real(r)))
                }
            }
            _ => {
                self.num(&x)?;
                self.num(&y)?;
                unreachable!()
            }
        }
    }

    // ----- standard procedures -----

    fn arg_num(&mut self, args: &[Arg], k: usize, env: &Rc<Frame>) -> R<f64> {
        let v = self.eval(&args[k], env)?;
        self.num(&v)
    }

    fn arg_int(&mut self, args: &[Arg], k: usize, env: &Rc<Frame>) -> R<i64> {
        let v = self.eval(&args[k], env)?;
        self.to_int(v)
    }

    fn arg_str(&mut self, args: &[Arg], k: usize, env: &Rc<Frame>) -> R<Rc<str>> {
        match self.eval(&args[k], env)? {
            Value::Str(s) => Ok(s),
            v => self.fail(format!("a string expected, but found {}", v.kind())),
        }
    }

    fn arg_loc(&mut self, args: &[Arg], k: usize, env: &Rc<Frame>, who: &str) -> R<Loc> {
        match &*args[k] {
            Ex::Load(v) => self.locate_var(v, env),
            Ex::Index(v, subs) => self.locate_index(v, subs, env),
            _ => self.fail(format!("the last parameter of {} must be a variable", who)),
        }
    }

    fn write(&mut self, s: &str) -> R<()> {
        if self.out.write_all(s.as_bytes()).is_err() {
            return self.fail("output failed");
        }
        for c in s.chars() {
            self.col = if c == '\n' { 0 } else { self.col + 1 };
        }
        Ok(())
    }

    /// Output of the X1 library: a new line first if `width` more symbols
    /// would not fit on the line of 144 positions.
    fn write_item(&mut self, s: &str) -> R<()> {
        if self.col + s.chars().count() > LINE_WIDTH {
            self.write("\n")?;
        }
        self.write(s)
    }

    fn read_char(&mut self) -> R<Option<char>> {
        if self.inpos >= self.inbuf.len() {
            let _ = self.out.flush();
            let mut line = String::new();
            match self.input.read_line(&mut line) {
                Ok(0) => return Ok(None),
                Ok(_) => {}
                Err(_) => return self.fail("input failed"),
            }
            self.inbuf = line.chars().collect();
            self.inpos = 0;
        }
        let c = self.inbuf[self.inpos];
        self.inpos += 1;
        Ok(Some(c))
    }

    fn peek_char(&mut self) -> R<Option<char>> {
        let c = self.read_char()?;
        if c.is_some() {
            self.inpos -= 1;
        }
        Ok(c)
    }

    /// The next number on the input, as text.
    fn read_number(&mut self, who: &str) -> R<String> {
        while let Some(c) = self.peek_char()? {
            if c.is_whitespace() {
                self.read_char()?;
            } else {
                break;
            }
        }
        let mut s = String::new();
        while let Some(c) = self.peek_char()? {
            if c.is_whitespace() || c == ',' || c == ';' {
                break;
            }
            s.push(c);
            self.read_char()?;
        }
        if let Some(',' | ';') = self.peek_char()? {
            self.read_char()?;
        }
        if s.is_empty() {
            return self.fail(format!("{}: the input is exhausted", who));
        }
        Ok(s)
    }

    #[inline(never)]
    fn builtin(&mut self, b: Builtin, args: &[Arg], env: &Rc<Frame>) -> R<Value> {
        use Builtin::*;
        Ok(match b {
            // On the X1 abs has the type of its argument (property 6).
            Abs if self.x1() => match self.eval(&args[0], env)? {
                Value::Int(i) => Value::Int(i.abs()),
                v => Value::Real(self.num(&v)?.abs()),
            },
            Abs => Value::Real(self.arg_num(args, 0, env)?.abs()),
            Iabs => match self.arg_int(args, 0, env)?.checked_abs() {
                Some(n) => Value::Int(n),
                None => return self.fail("integer overflow"),
            },
            Sign => {
                let v = self.eval(&args[0], env)?;
                Value::Int(self.sign(&v)?)
            }
            // On the X1 sqrt and ln take the absolute value (property 6).
            Sqrt if self.x1() => Value::Real(self.arg_num(args, 0, env)?.abs().sqrt()),
            Ln if self.x1() => {
                let x = self.arg_num(args, 0, env)?.abs();
                if x == 0.0 {
                    return self.fail("ln(0) is undefined");
                }
                Value::Real(x.ln())
            }
            Sqrt => {
                let x = self.arg_num(args, 0, env)?;
                if x < 0.0 {
                    return self.fail(format!("sqrt({}) is undefined", format_real(x)));
                }
                Value::Real(x.sqrt())
            }
            Sin => Value::Real(self.arg_num(args, 0, env)?.sin()),
            Cos => Value::Real(self.arg_num(args, 0, env)?.cos()),
            Arctan => Value::Real(self.arg_num(args, 0, env)?.atan()),
            Ln => {
                let x = self.arg_num(args, 0, env)?;
                if x <= 0.0 {
                    return self.fail(format!("ln({}) is undefined", format_real(x)));
                }
                Value::Real(x.ln())
            }
            Exp => {
                let x = self.arg_num(args, 0, env)?;
                self.real(x.exp())?
            }
            Entier => {
                let x = self.arg_num(args, 0, env)?;
                let f = x.floor();
                if !(-9.2e18..9.2e18).contains(&f) {
                    return self.fail(format!("entier({}) is too large for an integer", format_real(x)));
                }
                if self.x1() && f.abs() > X1_MAXINT as f64 {
                    return self.x1_stop("1–7", "in function entier an integer exceeds the integer capacity");
                }
                Value::Int(f as i64)
            }
            Length => Value::Int(self.arg_str(args, 0, env)?.chars().count() as i64),
            Outstring => {
                self.arg_int(args, 0, env)?;
                let s = self.arg_str(args, 1, env)?;
                self.write(&s)?;
                Value::Undef
            }
            Outinteger => {
                self.arg_int(args, 0, env)?;
                let i = self.arg_int(args, 1, env)?;
                self.write(&format!("{} ", i))?;
                Value::Undef
            }
            Outreal => {
                self.arg_int(args, 0, env)?;
                let x = self.arg_num(args, 1, env)?;
                let x = if self.x1() && x.abs() == x1_zero() { 0.0 } else { x };
                self.write(&format!("{} ", format_real(x)))?;
                Value::Undef
            }
            Outboolean => {
                self.arg_int(args, 0, env)?;
                let b = self.eval_bool(&args[1], env)?;
                self.write(if b { "true " } else { "false " })?;
                Value::Undef
            }
            Outchar => {
                self.arg_int(args, 0, env)?;
                let s = self.arg_str(args, 1, env)?;
                let i = self.arg_int(args, 2, env)?;
                match s.chars().nth((i - 1).max(0) as usize).filter(|_| i >= 1) {
                    Some(c) => self.write(&c.to_string())?,
                    None => return self.fail(format!("outchar: there is no character {} in the string", i)),
                }
                Value::Undef
            }
            Outterminator => {
                self.arg_int(args, 0, env)?;
                self.write(" ")?;
                Value::Undef
            }
            Inchar => {
                self.arg_int(args, 0, env)?;
                let s = self.arg_str(args, 1, env)?;
                let loc = self.arg_loc(args, 2, env, "inchar")?;
                let v = match self.read_char()? {
                    Some(c) => s.chars().position(|d| d == c).map(|p| p as i64 + 1).unwrap_or(0),
                    None => return self.fail("inchar: the input is exhausted"),
                };
                self.store(&loc, Value::Int(v))?;
                Value::Undef
            }
            Ininteger => {
                self.arg_int(args, 0, env)?;
                let loc = self.arg_loc(args, 1, env, "ininteger")?;
                let s = self.read_number("ininteger")?;
                match s.trim_start_matches('+').parse::<i64>() {
                    Ok(n) => self.store(&loc, Value::Int(n))?,
                    Err(_) => return self.fail(format!("ininteger: {} is not an integer", s)),
                }
                Value::Undef
            }
            Inreal => {
                self.arg_int(args, 0, env)?;
                let loc = self.arg_loc(args, 1, env, "inreal")?;
                let s = self.read_number("inreal")?;
                let t: String = s.chars().map(|c| if c == '⏨' || c == '#' { 'e' } else { c }).collect();
                match t.trim_start_matches('+').parse::<f64>() {
                    Ok(x) if x.is_finite() => self.store(&loc, Value::Real(x))?,
                    _ => return self.fail(format!("inreal: {} is not a number", s)),
                }
                Value::Undef
            }
            Stop => return Err(Signal::Stop),
            Fault => {
                let s = self.arg_str(args, 0, env)?;
                let x = self.arg_num(args, 1, env)?;
                return self.fail(format!("fault: {} {}", s, format_real(x)));
            }
            Read => {
                let s = self.read_number("read")?;
                let t: String = s.chars().map(|c| if c == '⏨' || c == '#' { 'e' } else { c }).collect();
                let t = t.trim_start_matches('+');
                match (t.parse::<i64>(), t.parse::<f64>()) {
                    (Ok(n), _) => self.int(n)?,
                    (_, Ok(x)) if x.is_finite() => Value::Real(x),
                    _ => return self.x1_stop("1–3", &format!("read: {} is not a number", s)),
                }
            }
            // The layouts below are those of the provisional library manual
            // of the X8 system, section 7.3, as translated in
            // sergev/x1-algol-compiler (doc/library-procedures.md). No
            // description of the X1's own survives in the sources found.
            Print => {
                // Dijkstra's course (2nd edition, 1962): an integer is typed
                // in at most eight decimals with non-significant zeros as
                // spaces; a real in floating notation, and as "the integer 0
                // or 'inf'" if its decimal exponent would exceed 600; with a
                // sign in all cases. The layout follows the type, as in MR 54
                // ("if type(x) = integer then FIXT(8, 0, x)"). The column of
                // 21 positions is the X8's; the X1's tab stops are not known.
                let v = self.eval(&args[0], env)?;
                let text = match v {
                    Value::Int(i) => format!("{:<21}", fixt(8, 0, i as f64, true)),
                    v => flot(13, 3, self.num(&v)?),
                };
                self.write_item(&text)?;
                Value::Undef
            }
            Tab => {
                let to = (self.col + 2).div_ceil(8) * 8;
                if to > LINE_WIDTH {
                    self.write("\n")?;
                } else {
                    let n = to - self.col;
                    self.write(&" ".repeat(n))?;
                }
                Value::Undef
            }
            Nlcr => {
                self.write("\n")?;
                Value::Undef
            }
            Space => {
                let v = self.eval(&args[0], env)?;
                let Value::Int(n) = v else {
                    return self.x1_stop("1–11", "the actual parameter of procedure SPACE is not of integer type");
                };
                for _ in 0..n.max(0) {
                    if self.col >= LINE_WIDTH {
                        self.write("\n")?;
                    }
                    self.write(" ")?;
                }
                Value::Undef
            }
            Printtext => {
                let s = self.arg_str(args, 0, env)?;
                for c in s.chars() {
                    if self.col >= LINE_WIDTH && c != '\n' {
                        self.write("\n")?;
                    }
                    self.write(&c.to_string())?;
                }
                Value::Undef
            }
            Even => {
                let n = self.arg_int(args, 0, env)?;
                Value::Int(if n % 2 == 0 { 1 } else { -1 })
            }
            Sum => {
                // real procedure SUM(i, a, b, x); value b; integer i, a, b;
                // real x; begin real s; s := 0; for i := a step 1 until b
                // do s := s + x; SUM := s end (manual, 7.5.1).
                let hi = self.arg_int(args, 2, env)?;
                let loc = self.arg_loc(args, 0, env, "SUM")?;
                let lo = self.arg_int(args, 1, env)?;
                self.store(&loc, Value::Int(lo))?;
                let mut total = Value::Real(0.0);
                loop {
                    self.tick()?;
                    let i = self.load_loc(&loc);
                    let i = self.to_int(i)?;
                    if i > hi {
                        break;
                    }
                    let t = self.eval(&args[3], env)?;
                    total = self.arith(BOp::Add, total, t)?;
                    let loc = self.arg_loc(args, 0, env, "SUM")?;
                    self.store(&loc, Value::Int(i + 1))?;
                }
                total
            }
            Flot => {
                // The X1's FLOT has two parameters (Zonneveld's program in
                // appendix B of Kruseman Aretz 2003 calls FLOT(12, x)); the
                // X8's has three. The X1's exponent "was here fixed to two"
                // decimals (X1–X8 comparison, appendix A, note 3).
                let n = self.arg_int(args, 0, env)?;
                let x = self.arg_num(args, 1, env)?;
                self.write_item(&flot(n, 2, x))?;
                Value::Undef
            }
            Fixt | Absfixt => {
                let n = self.arg_int(args, 0, env)?;
                let m = self.arg_int(args, 1, env)?;
                let x = self.arg_num(args, 2, env)?;
                self.write_item(&fixt(n, m, x, b == Fixt))?;
                Value::Undef
            }
            Maxreal => Value::Real(f64::MAX),
            Minreal => Value::Real(f64::MIN_POSITIVE),
            Maxint => Value::Int(i64::MAX),
            Epsilon => Value::Real(f64::EPSILON),
        })
    }
}

const LINE_WIDTH: usize = 144;

/// FLOT(n, m, x) (X8 manual, 7.3.1.10): the sign of x, a point, an n-digit
/// mantissa between .1 and 1, ⏨, the sign of the exponent, the exponent in
/// m digits with leading zeros as spaces, and a space. Out of range, it is
/// FLOT(13, 3, x).
fn flot(n: i64, m: i64, x: f64) -> String {
    if !(1..=13).contains(&n) || !(1..=3).contains(&m) {
        return flot(13, 3, x);
    }
    let (n, m) = (n as usize, m as usize);
    // A number whose decimal exponent would exceed 600 in modulus, the X1's
    // zero and its greatest real among them, is typed as "the integer 0 or
    // 'inf'" (Dijkstra's course), with its sign, at the right of the field:
    // a mass of 0 in Zonneveld's output of 1964 (MC Tract 8, p. 106) is
    // `+                  0`.
    let width = n + m + 4;
    if x.abs() <= X1_REAL_MIN || x.abs() >= x1_real_max() {
        let word = if x.abs() <= X1_REAL_MIN { "0" } else { "inf" };
        return format!("{}{:>w$} ", if x < 0.0 { '-' } else { '+' }, word, w = width - 1);
    }
    // x = 0.d1d2... × 10^e, rounded to n digits.
    let s = format!("{:.*e}", n - 1, x.abs());
    let (mant, exp) = s.split_once('e').unwrap();
    let digits = mant.replace('.', "");
    let e = exp.parse::<i32>().unwrap() + 1;
    let ee = e.unsigned_abs().to_string();
    if ee.len() > m {
        return if m == 3 && n == 13 { format!("{:e} ", x) } else { flot(13, 3, x) };
    }
    format!(
        "{}.{}⏨{}{:>width$} ",
        if x < 0.0 { '-' } else { '+' },
        digits,
        if e < 0 { '-' } else { '+' },
        ee,
        width = m
    )
}

/// ABSFIXT(n, m, x) (X8 manual, 7.3.1.8): a space, |x| with n digits
/// before the point and m after (no point if m = 0), leading zeros as
/// spaces except the units zero when m = 0, and a space. FIXT puts the
/// sign of x in place of the first space (the X1's output of 1964; the X8
/// manual puts it beside the first digit). Out of range, FLOT(13, 3, x).
fn fixt(n: i64, m: i64, x: f64, signed: bool) -> String {
    if n < 0 || m < 0 || n + m > 21 {
        return flot(13, 3, x);
    }
    let (n, m) = (n as usize, m as usize);
    let text = format!("{:.*}", m, x.abs());
    let (int_part, frac) = match text.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (text.clone(), None),
    };
    let int_part = if int_part == "0" && m > 0 { String::new() } else { int_part };
    if int_part.len() > n {
        return flot(13, 3, x);
    }
    let mut out = format!(" {:>width$}", int_part, width = n);
    if let Some(f) = frac {
        out.push('.');
        out.push_str(&f);
    }
    out.push(' ');
    if signed {
        // The sign stands in the first column, before the spaces of the
        // leading zeros: `+ 3.429474152`, `- .049534455` in Zonneveld's
        // output of 1964 (MC Tract 8, p. 103), and MR 54: "Typt teken, n
        // decimale cijfers van geheel deel van x, ...".
        out.replace_range(0..1, if x < 0.0 { "-" } else { "+" });
    }
    out
}
