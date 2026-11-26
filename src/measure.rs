//! What a running program holds, for measurement only (`--measure`).
//!
//! The counters are moved when activation records, arrays and name
//! parameters are made and dropped, and when procedures are entered and
//! left. Nothing here is read by the interpreter, so no result depends on
//! it. The counters are process-wide, and are meant for one program run by
//! the command line at a time.

use std::sync::atomic::{AtomicI64, Ordering::Relaxed};

macro_rules! counters {
    ($($name:ident: $doc:literal),* $(,)?) => {
        /// The quantities measured, in order.
        pub const FIELDS: &[(&str, &str)] = &[$((stringify!($name), $doc)),*];
        #[allow(non_upper_case_globals)]
        mod cells {
            use std::sync::atomic::AtomicI64;
            $(pub static $name: [AtomicI64; 2] = [AtomicI64::new(0), AtomicI64::new(0)];)*
        }
        /// (name, current, peak) for each quantity.
        pub fn snapshot() -> Vec<(&'static str, i64, i64)> {
            vec![$((stringify!($name), cells::$name[0].load(Relaxed), cells::$name[1].load(Relaxed))),*]
        }
        $(#[allow(dead_code)] fn $name(d: i64) { add(&cells::$name, d) })*
    };
}

counters! {
    frames: "activation records alive (blocks and procedure calls, including those kept by procedures and name parameters)",
    slots: "variables in the live activation records",
    arrays: "arrays alive",
    array_elements: "elements of the live arrays",
    names: "name parameters (call by name) alive, each an expression with its environment",
    call_depth: "procedure calls in progress",
    frames_made: "activation records made, in all",
    calls: "procedure calls made, in all",
    name_uses: "uses of a name parameter, in all",
    name_evaluations: "evaluations of a name parameter's expression, in all",
    name_first_evaluations: "first evaluations: an actual parameter evaluated for the first time",
    name_repeat_evaluations: "repeat evaluations: the same actual parameter, of the same call, evaluated again",
}

fn add(c: &[AtomicI64; 2], d: i64) {
    let now = c[0].fetch_add(d, Relaxed) + d;
    c[1].fetch_max(now, Relaxed);
}

pub fn frame_made(n: usize) {
    frames(1);
    slots(n as i64);
    frames_made(1);
}

pub fn frame_dropped(n: usize) {
    frames(-1);
    slots(-(n as i64));
}

pub fn array_made(n: usize) {
    arrays(1);
    array_elements(n as i64);
}

pub fn array_dropped(n: usize) {
    arrays(-1);
    array_elements(-(n as i64));
}

pub fn name_made() {
    names(1);
}

pub fn name_dropped() {
    names(-1);
}

pub fn name_used() {
    name_uses(1);
}

/// An actual parameter called by name is evaluated. Its own count tells a
/// first evaluation from a repeat of the same parameter of the same call.
pub fn name_evaluated(count: &std::cell::Cell<u64>) {
    name_evaluations(1);
    if count.get() == 0 {
        name_first_evaluations(1);
    } else {
        name_repeat_evaluations(1);
    }
    count.set(count.get() + 1);
}

/// Held while a procedure call is in progress.
pub struct Call;

impl Call {
    pub fn enter() -> Call {
        call_depth(1);
        calls(1);
        Call
    }
}

impl Drop for Call {
    fn drop(&mut self) {
        call_depth(-1);
    }
}

pub fn made_array(a: crate::value::Array) -> crate::value::Array {
    array_made(a.data.borrow().len());
    a
}

pub fn made_thunk(t: crate::value::Thunk) -> crate::value::Thunk {
    name_made();
    t
}
