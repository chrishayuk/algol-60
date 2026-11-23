//! The X1 dialect (`--x1`): each of the documented ways in which the
//! Dijkstra–Zonneveld system for the Electrologica X1 departed from the
//! Revised Report, with the same program run both ways.
//!
//! "Property n" is the n-th of the "special properties of the
//! MC-Algol-system" listed in the user manual of December 1962, as given
//! by Kruseman Aretz (2003), section 1.3. "Stop n" is a run-time stop of
//! appendix A of the same report.

use algol60::{Dialect, Options};

fn run(dialect: Dialect, body: &str) -> String {
    let src = format!("begin {} end", body);
    let (out, r) = algol60::run_source_with(&src, "", Options { max_steps: Some(10_000_000), dialect });
    match r {
        Ok(()) => out,
        Err(e) => format!("{}! {}", out, e),
    }
}

fn both(body: &str) -> (String, String) {
    (run(Dialect::Report, body), run(Dialect::X1, body))
}

/// The X1 manual forbade it, but the compiler did not check: a warning.
fn warned(body: &str, property: u32) {
    let src = format!("begin {} end", body);
    assert!(algol60::check(&src, Dialect::Report).unwrap().is_empty(), "the Report warns: {}", body);
    let w = algol60::check(&src, Dialect::X1).unwrap_or_else(|e| panic!("the X1 dialect refuses {}: {}", body, e));
    assert!(w.iter().any(|w| w.contains(&format!("(property {})", property))), "{} gave: {:?}", body, w);
}

#[test]
fn property_11_the_step_is_evaluated_once_each_time_round() {
    let p = "integer nb, i; integer procedure b; begin nb := nb + 1; b := 1 end;
             for i := 1 step b until 10 do ; outinteger(1, nb)";
    assert_eq!(both(p), ("21 ".into(), "11 ".into()));
}

#[test]
fn property_11_the_step_is_evaluated_before_the_initial_value_is_assigned() {
    let p = "integer i, first, calls;
             integer procedure b; begin if calls = 0 then first := i; calls := calls + 1; b := 1 end;
             i := 7; calls := 0; for i := 1 step b until 1 do ; outinteger(1, first)";
    assert_eq!(both(p), ("1 ".into(), "7 ".into()));
}

#[test]
fn the_limit_is_evaluated_at_every_test_in_both() {
    let p = "integer nc, i; integer procedure c; begin nc := nc + 1; c := 10 end;
             for i := 1 step 1 until c do ; outinteger(1, nc)";
    assert_eq!(both(p), ("11 ".into(), "11 ".into()));
}

#[test]
fn property_10_an_undefined_switch_designator_is_not_a_dummy_statement() {
    let (r, x) = both("switch s := a; go to s[2]; outstring(1, \"fell through\"); a:");
    assert_eq!(r, "fell through");
    assert!(x.contains("undefined on the X1 (property 10)"), "{}", x);
}

#[test]
fn property_6_abs_has_the_type_of_its_argument() {
    let (r, x) = both("outinteger(1, abs(-7) ÷ 2)");
    assert!(r.contains("'÷' is defined only for integers"), "{}", r);
    assert_eq!(x, "3 ");
}

#[test]
fn property_6_sqrt_and_ln_take_the_absolute_value() {
    let (r, x) = both("outreal(1, sqrt(-4))");
    assert!(r.contains("sqrt(-4) is undefined"), "{}", r);
    assert_eq!(x, "2 ");
    assert_eq!(run(Dialect::X1, "outreal(1, ln(-1))"), "0 ");
}

#[test]
fn property_18_a_function_value_keeps_the_type_of_its_arithmetic() {
    let p = "integer procedure half(y); value y; real y; half := y / 2; outreal(1, half(3))";
    assert_eq!(both(p), ("2 ".into(), "1.5 ".into()));
}

#[test]
fn property_22_values_are_assigned_in_the_order_of_specification() {
    let p = "integer procedure note(k); value k; integer k; begin outinteger(1, k); note := k end;
             procedure order(a, b); value a, b; integer b; integer a; ;
             order(note(1), note(2))";
    assert_eq!(both(p), ("1 2 ".into(), "2 1 ".into()));
}

#[test]
fn property_3_only_nine_symbols_of_an_identifier_count() {
    let p = "integer abcdefghiX; abcdefghiX := 0; begin integer abcdefghiY; abcdefghiX := 1 end; outinteger(1, abcdefghiX)";
    assert_eq!(both(p), ("1 ".into(), "0 ".into()));
    assert!(run(Dialect::X1, "integer abcdefghi1, abcdefghi2;").contains("declared twice"));
}

#[test]
fn property_4_zero_is_an_integer_and_large_whole_numbers_are_real() {
    assert_eq!(both("outinteger(1, 0.0 ÷ 1)").1, "0 ");
    assert!(both("outinteger(1, 0.0 ÷ 1)").0.contains("defined only for integers"));
    assert_eq!(both("outinteger(1, 67108863 ÷ 2)"), ("33554431 ".into(), "33554431 ".into()));
    assert!(run(Dialect::X1, "outinteger(1, 67108864 ÷ 2)").contains("defined only for integers"));
}

#[test]
fn stop_1_1_integers_have_26_bits_and_a_sign() {
    let p = "integer i; i := 67108863; i := i + 1; outinteger(1, i)";
    let (r, x) = both(p);
    assert_eq!(r, "67108864 ");
    assert!(x.contains("X1 stop 1–1"), "{}", x);
}

#[test]
fn stop_1_2_a_lower_bound_above_the_upper_is_an_error() {
    let p = "integer n; n := 0; begin integer array a[1:n]; outstring(1, \"ok\") end";
    let (r, x) = both(p);
    assert_eq!(r, "ok");
    assert!(x.contains("X1 stop 1–2"), "{}", x);
}

#[test]
fn reals_are_stored_with_a_mantissa_of_40_bits() {
    // 1/3 is computed with 52 bits on the stack and stored with 40.
    let (r, x) = both("real x; x := 1 / 3; outreal(1, x - 1 / 3)");
    assert_eq!(r, "0 ");
    assert_ne!(x, "0 ");
    // A real constant is stored like a variable, so 0.1 - 0.1 stored is 0
    // (x1algol.pas, look_for_constant; the real compiler on x1sim agrees).
    assert_eq!(run(Dialect::X1, "real x; x := 0.1; outreal(1, x - 0.1)"), "0 ");
    // Exact in 40 bits: no difference.
    assert_eq!(run(Dialect::X1, "real x; x := 0.75; outreal(1, x - 0.75)"), "0 ");
}

// ----- what the X1 manual forbade, unchecked by its compiler -----

#[test]
fn property_8_no_numeric_labels_and_the_compiler_fails_on_them() {
    let e = algol60::check("begin go to 10; 10: end", Dialect::X1).unwrap_err();
    assert!(e.contains("property 8"), "{}", e);
}

#[test]
fn property_12_no_conditional_statement_after_do() {
    warned("integer i; for i := 1 do if true then i := 2", 12);
}

#[test]
fn property_13_only_commas_between_parameters() {
    warned("procedure p(a, b); integer a, b; ; p(1) plus: (2)", 13);
}

#[test]
fn property_15_declarations_in_order() {
    warned("procedure p; ; integer i;", 15);
    warned("integer array a[1:2]; integer i;", 15);
    warned("procedure p(a, l); integer array a; integer l; ;", 15);
}

#[test]
fn property_17_constant_bounds_in_the_outermost_block() {
    warned("integer n; integer array a[1:n];", 17);
    warned("integer n; begin own integer array a[1:n]; end", 17);
    assert!(algol60::check("begin integer n; begin integer array a[1:n]; end end", Dialect::X1).unwrap().is_empty());
}

#[test]
fn property_19_every_parameter_is_specified() {
    warned("procedure p(a); a := 1;", 19);
}

#[test]
fn property_21_no_label_in_the_value_list() {
    warned("procedure p(l); value l; label l; go to l;", 21);
}

#[test]
fn property_5_standard_functions_are_not_statements() {
    warned("sqrt(2)", 5);
}

#[test]
fn chapter_7_operators_are_not_actual_parameters() {
    let src = "begin real procedure twice(f, x); real x; real procedure f; twice := f(f(x)); outreal(1, twice(sqrt, 16)) end";
    assert!(algol60::check(src, Dialect::Report).unwrap().is_empty());
    assert!(algol60::check(src, Dialect::X1).unwrap()[0].contains("could not be an actual parameter"));
}

#[test]
fn knuths_1964_program_breaks_the_rule_that_parameters_are_specified() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/man-or-boy-1964.alg")).unwrap();
    assert!(algol60::check(&src, Dialect::Report).unwrap().is_empty());
    let w = algol60::check(&src, Dialect::X1).unwrap();
    assert!(w.iter().any(|w| w.contains("(property 19)")), "{:?}", w);
}

#[test]
fn then_if_is_accepted_with_a_warning_and_else_goes_with_the_nearest_if() {
    let p = "integer i; i := 2; if i > 1 then if i = 1 then outstring(1, \"a\") else outstring(1, \"b\")";
    assert_eq!(run(Dialect::X1, p), "b");
    assert!(run(Dialect::Report, p).contains("must be enclosed in begin ... end"));
}

// ----- the X1 library -----

#[test]
fn the_x1_library() {
    // print: FIXT(13, 0, x) and six spaces for a whole number, else
    // FLOT(13, 3, x); 21 positions either way (X8 manual, 7.3.1.11).
    let p = "integer i; PRINTTEXT(`a'); SPACE(2); print(3); NLCR; print(SUM(i, 1, 4, i × i)); NLCR; print(1.5)";
    assert_eq!(
        run(Dialect::X1, p),
        "a              +3       \n           +30       \n+.1500000000000⏨+  1 "
    );
    assert!(run(Dialect::Report, p).contains("PRINTTEXT is not declared"));
}

#[test]
fn read_takes_the_next_number() {
    let src = "begin real a; integer b; a := read; b := read; print(a + b) end";
    let (out, r) = algol60::run_source_with(src, "2.5 4", Options { max_steps: None, dialect: Dialect::X1 });
    r.unwrap();
    assert_eq!(out, "+.6500000000000⏨+  1 ");
}

#[test]
fn zonnevelds_program_from_appendix_b_is_accepted() {
    // It ran on the X1, so the restrictions here must not refuse it.
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/historical/jaz164.alg")).unwrap();
    algol60::check(&src, Dialect::X1).unwrap();
    // SUM is the X1's, not the Report's.
    assert!(algol60::check(&src, Dialect::Report).unwrap_err().contains("SUM is not declared"));
}

#[test]
fn fixt_and_absfixt_follow_the_manual() {
    // X8 manual 7.3.1.8-9. The cases are those of sergev/x1-algol-compiler
    // (tests/print_test.cpp), whose digits agree; its sign is placed in the
    // first column, the manual's beside the first digit, as here.
    let p = "procedure pr(n, m, x); value n, m, x; integer n, m; real x;
             begin PRINTTEXT(`/'); ABSFIXT(n, m, x); FIXT(n, m, x); PRINTTEXT(`/'); NLCR end;
             pr(4, 0, -56.789); pr(1, 20, 8 / 7); pr(0, 21, -1 / 7); pr(2, 0, 0.0); pr(4, 1, 0.5)";
    assert_eq!(
        run(Dialect::X1, p),
        "/   57   -57 /\n/ 1.14285714285688300151 +1.14285714285688300151 /\n/ .142857142857110375189 -.142857142857110375189 /\n/  0  +0 /\n/     .5     +.5 /\n"
    );
}

#[test]
fn flot() {
    let p = "FLOT(13, -1⏨-111 / 7); FLOT(4, 9.9999); FLOT(1, 0)";
    assert_eq!(run(Dialect::X1, p), "-.1428571428571⏨-111 +.1000⏨+  2 +.0⏨+  0 ");
}

#[test]
fn zonnevelds_outer_planets() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/historical/");
    let read = |f: &str| std::fs::read_to_string(format!("{}{}", dir, f)).unwrap();
    let (out, r) = algol60::run_source_with(&read("jaz164.alg"), &read("jaz164.in"), Options { max_steps: None, dialect: Dialect::X1 });
    r.unwrap();
    assert_eq!(out, read("jaz164.x1.out"));
}
