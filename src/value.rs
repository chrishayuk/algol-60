//! Run-time values and frames.

use crate::ast::Type;
use crate::builtins::Builtin;
use crate::compile::{Ex, LabelId};
use std::cell::RefCell;
use std::rc::Rc;

/// The storage of one block or procedure activation. `parent` is the
/// static link: the frame of the block the declaration appears in.
pub struct Frame {
    pub id: u64,
    pub parent: Option<Rc<Frame>>,
    pub slots: RefCell<Vec<Value>>,
}

/// A procedure together with the frame it was declared in.
pub struct Closure {
    pub id: u32,
    pub env: Rc<Frame>,
}

pub struct SwitchVal {
    pub id: u32,
    pub env: Rc<Frame>,
}

/// An actual parameter called by name: the expression and the frame of the
/// call it is to be evaluated in, each time it is used (4.7.3.2).
pub struct Thunk {
    pub ex: Rc<Ex>,
    pub env: Rc<Frame>,
    /// how many times this actual parameter has been evaluated (measurement only)
    pub evaluations: std::cell::Cell<u64>,
}

pub struct Array {
    pub ty: Type,
    pub bounds: Vec<(i64, i64)>,
    pub data: RefCell<Vec<Value>>,
}

/// A label of one particular activation of its block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabelVal {
    pub frame: u64,
    pub id: LabelId,
}

#[derive(Clone)]
pub enum Value {
    Undef,
    Int(i64),
    Real(f64),
    Bool(bool),
    Str(Rc<str>),
    Array(Rc<Array>),
    Proc(Rc<Closure>),
    Builtin(Builtin),
    Switch(Rc<SwitchVal>),
    Label(LabelVal),
    Name(Rc<Thunk>),
}

impl Value {
    pub fn zero(ty: Type) -> Value {
        match ty {
            Type::Integer => Value::Int(0),
            Type::Real => Value::Real(0.0),
            Type::Boolean => Value::Bool(false),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Value::Undef => "an undefined value",
            Value::Int(_) => "an integer",
            Value::Real(_) => "a real",
            Value::Bool(_) => "a Boolean",
            Value::Str(_) => "a string",
            Value::Array(_) => "an array",
            Value::Proc(_) | Value::Builtin(_) => "a procedure",
            Value::Switch(_) => "a switch",
            Value::Label(_) => "a label",
            Value::Name(_) => "a name",
        }
    }
}

/// `%.15g`, as C prints it.
pub fn format_real(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let s = format!("{:.14e}", x);
    let (mant, exp) = s.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if !(-5..15).contains(&exp) {
        let mant = trim_zeros(mant);
        format!("{}e{}{:02}", mant, if exp < 0 { '-' } else { '+' }, exp.abs())
    } else {
        let decimals = (14 - exp).max(0) as usize;
        trim_zeros(&format!("{:.*}", decimals, x)).to_string()
    }
}

fn trim_zeros(s: &str) -> &str {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        s
    }
}

/// x with its mantissa rounded to `bits` bits.
pub fn round_mantissa(x: f64, bits: i32) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let e = x.abs().log2().floor() as i32 + 1;
    // The scale 2^(bits − e) is applied in two halves, so that it stays
    // finite for x near the smallest normal number.
    let k = bits - e;
    let (s1, s2) = (2f64.powi(k / 2), 2f64.powi(k - k / 2));
    (x * s1 * s2).round() / s1 / s2
}

impl Drop for Frame {
    fn drop(&mut self) {
        crate::measure::frame_dropped(self.slots.get_mut().len());
    }
}

impl Drop for Array {
    fn drop(&mut self) {
        crate::measure::array_dropped(self.data.get_mut().len());
    }
}

impl Drop for Thunk {
    fn drop(&mut self) {
        crate::measure::name_dropped();
    }
}
