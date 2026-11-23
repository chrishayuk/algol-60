//! The rules of the Revised Report one at a time, each with a program
//! small enough to work out by hand, and the programs the Report's syntax
//! refuses.

fn run(body: &str) -> String {
    let src = format!("begin {} end", body);
    let (out, r) = algol60::run_source(&src, "", Some(10_000_000));
    match r {
        Ok(()) => out,
        Err(e) => format!("{}! {}", out, e),
    }
}

fn error(body: &str) -> String {
    let src = format!("begin {} end", body);
    match algol60::run_source(&src, "", Some(10_000_000)).1 {
        Err(e) => e,
        Ok(()) => panic!("no error for: {}", body),
    }
}

// ----- Knuth's man or boy test -----

const MAN_OR_BOY: &str = "
  real procedure A(k, x1, x2, x3, x4, x5); value k; integer k;
  begin real procedure B; begin k := k - 1; B := A := A(k, B, x1, x2, x3, x4) end;
    if k <= 0 then A := x4 + x5 else B
  end;";

#[test]
fn man_or_boy_agrees_with_knuth_for_k_up_to_20() {
    // OEIS A132343. In 1964 Knuth guessed -121 for k = 10, by hand; it is -67.
    let want = [
        1, 0, -2, 0, 1, 0, 1, -1, -10, -30, -67, -138, -291, -642, -1446, -3250, -7244, -16065, -35601, -78985, -175416,
    ];
    let prog = format!("{} integer k; for k := 0 step 1 until 20 do outreal(1, A(k, 1, -1, -1, 1, 0))", MAN_OR_BOY);
    let (out, r) = algol60::run_source(&format!("begin {} end", prog), "", None);
    r.unwrap();
    let got: Vec<i64> = out.split_whitespace().map(|s| s.parse().unwrap()).collect();
    assert_eq!(got, want);
}

// ----- call by name -----

#[test]
fn jensens_device() {
    let p = "real procedure Sum(k, lo, hi, t); value lo, hi; integer k, lo, hi; real t;
             begin real s; s := 0; for k := lo step 1 until hi do s := s + t; Sum := s end;
             integer i; outreal(1, Sum(i, 1, 4, i × i))";
    assert_eq!(run(p), "30 ");
}

#[test]
fn a_name_parameter_is_evaluated_at_every_use() {
    let p = "integer n; integer procedure f; begin n := n + 1; f := n end;
             integer procedure three(x); integer x; three := x + x + x;
             outinteger(1, three(f)); outinteger(1, n)";
    assert_eq!(run(p), "6 3 ");
}

#[test]
fn a_name_parameter_that_is_never_used_is_never_evaluated() {
    let p = "integer procedure ignore(x); integer x; ignore := 1;
             outinteger(1, ignore(1 ÷ 0))";
    assert_eq!(run(p), "1 ");
}

#[test]
fn a_value_parameter_is_evaluated_once_at_the_call() {
    let p = "integer n; integer procedure f; begin n := n + 1; f := n end;
             integer procedure three(x); value x; integer x; three := x + x + x;
             outinteger(1, three(f)); outinteger(1, n)";
    assert_eq!(run(p), "3 1 ");
}

#[test]
fn assignment_to_a_name_parameter_assigns_to_the_actual_variable() {
    let p = "procedure set(x); integer x; x := 7; integer i; integer array a[1:2];
             set(i); set(a[2]); outinteger(1, i); outinteger(1, a[2])";
    assert_eq!(run(p), "7 7 ");
}

#[test]
fn the_subscript_of_a_name_parameter_is_evaluated_again() {
    let p = "procedure swap(x, y); integer x, y; begin integer t; t := x; x := y; y := t end;
             integer i; integer array a[1:5]; i := 1; a[1] := 5; swap(i, a[i]);
             outinteger(1, i); outinteger(1, a[1]); outinteger(1, a[5])";
    assert_eq!(run(p), "5 5 1 ");
}

#[test]
fn assigning_to_an_expression_called_by_name_is_an_error() {
    let e = error("procedure set(x); integer x; x := 7; integer i; set(i + 1)");
    assert!(e.contains("x is assigned to, but its actual parameter is an expression"), "{}", e);
}

#[test]
fn name_parameters_pass_through_several_procedures() {
    let p = "procedure inner(z); integer z; z := z + 1;
             procedure middle(y); integer y; begin inner(y); inner(y) end;
             procedure outer(x); integer x; middle(x);
             integer i; i := 10; outer(i); outinteger(1, i)";
    assert_eq!(run(p), "12 ");
}

#[test]
fn a_procedure_called_by_name_is_called_where_it_is_used() {
    let p = "real procedure twice(f, x); real procedure f; real x; twice := f(f(x));
             real procedure half(y); value y; real y; half := y / 2;
             outreal(1, twice(half, 10)); outreal(1, twice(sqrt, 16))";
    assert_eq!(run(p), "2.5 2 ");
}

#[test]
fn a_typed_procedure_without_parameters_is_called_when_its_name_is_evaluated() {
    let p = "integer n; integer procedure next; begin n := n + 1; next := n end;
             procedure show(x); integer x; begin outinteger(1, x); outinteger(1, x) end;
             show(next)";
    assert_eq!(run(p), "1 2 ");
}

// ----- the for statement (4.6) -----

#[test]
fn step_and_limit_are_evaluated_as_the_expansion_says() {
    let p = "integer nb, nc, i; integer procedure b; begin nb := nb + 1; b := 1 end;
             integer procedure c; begin nc := nc + 1; c := 3 end;
             for i := 1 step b until c do ; outinteger(1, nb); outinteger(1, nc)";
    // 4 tests (i = 1, 2, 3, 4) and 3 increments.
    assert_eq!(run(p), "7 4 ");
}

#[test]
fn a_negative_step_counts_down() {
    assert_eq!(run("integer i; for i := 5 step -2 until 0 do outinteger(1, i)"), "5 3 1 ");
}

#[test]
fn a_for_list_mixes_its_elements() {
    // The step-until element leaves i at 40, the value that failed the
    // test, so the while element starts from 41 and its body is not done.
    let p = "integer i; for i := 1, 2, 10 step 10 until 30, i + 1 while i < 43 do outinteger(1, i)";
    assert_eq!(run(p), "1 2 10 20 30 41 42 ");
}

#[test]
fn a_real_step_on_an_integer_variable_rounds_each_time() {
    assert_eq!(run("integer i; for i := 1 step 0.6 until 4 do outinteger(1, i)"), "1 2 3 4 ");
}

#[test]
fn going_to_a_label_inside_a_for_statement_from_outside_is_an_error() {
    let e = error("integer i; go to l; for i := 1 do l: outinteger(1, i)");
    assert!(e.contains("cannot be reached"), "{}", e);
}

#[test]
fn leaving_a_for_statement_by_go_to_keeps_the_variable() {
    assert_eq!(run("integer i; for i := 1 step 1 until 10 do if i = 4 then go to out; out: outinteger(1, i)"), "4 ");
}

// ----- assignment (4.2) -----

#[test]
fn left_part_subscripts_are_evaluated_before_the_expression() {
    let p = "integer i; integer array a[1:2]; integer procedure f; begin i := 2; f := 9 end;
             i := 1; a[i] := f; outinteger(1, a[1]); outinteger(1, a[2])";
    assert_eq!(run(p), "9 0 ");
}

#[test]
fn a_real_assigned_to_an_integer_is_rounded() {
    assert_eq!(run("integer i; i := 0.5; outinteger(1, i); i := -0.5; outinteger(1, i); i := -0.51; outinteger(1, i)"), "1 0 -1 ");
}

#[test]
fn a_boolean_cannot_be_assigned_to_a_number() {
    let e = error("integer i; i := true");
    assert!(e.contains("a Boolean cannot be assigned to a variable of type integer"), "{}", e);
}

// ----- expressions (3.3, 3.4) -----

#[test]
fn power_groups_from_the_left() {
    assert_eq!(run("outinteger(1, 2 ↑ 3 ↑ 2); outinteger(1, 2 ** 3 ** 2); outinteger(1, -3 ↑ 2)"), "64 64 -9 ");
}

#[test]
fn an_integer_to_a_negative_power_is_real() {
    assert_eq!(run("outreal(1, 2 ↑ (-2)); outreal(1, 10 ↑ (-1))"), "0.25 0.1 ");
}

#[test]
fn zero_to_the_zero_is_undefined() {
    assert!(error("outinteger(1, 0 ↑ 0)").contains("0↑0 is undefined"));
    assert!(error("outreal(1, (-8) ↑ 0.5)").contains("negative number to a real power"));
}

#[test]
fn integer_division_truncates_towards_zero() {
    assert_eq!(run("outinteger(1, 7 ÷ 2); outinteger(1, -7 ÷ 2); outinteger(1, 7 div (-2))"), "3 -3 -3 ");
}

#[test]
fn integer_division_is_only_for_integers() {
    assert!(error("outinteger(1, 7.0 ÷ 2)").contains("'÷' is defined only for integers"));
    assert!(error("outinteger(1, abs(-7) ÷ 2)").contains("'÷' is defined only for integers"));
}

#[test]
fn slash_always_gives_a_real() {
    assert_eq!(run("real x; x := 6 / 3; outreal(1, x); outreal(1, 1 / 3)"), "2 0.333333333333333 ");
}

#[test]
fn boolean_operators_have_the_reports_precedence() {
    // ¬ binds tighter than ∧, ∧ than ∨, ∨ than ⊃, ⊃ than ≡.
    let p = "outboolean(1, ¬ false ∧ false); outboolean(1, true ∨ false ∧ false);
             outboolean(1, false ⊃ false ≡ true); outboolean(1, ¬ 1 < 2)";
    assert_eq!(run(p), "false true true false ");
}

#[test]
fn boolean_operators_evaluate_both_operands() {
    let p = "integer n; Boolean procedure t; begin n := n + 1; t := true end;
             Boolean b; b := false ∧ t; b := true ∨ t; outinteger(1, n)";
    assert_eq!(run(p), "2 ");
}

#[test]
fn conditional_expressions() {
    assert_eq!(run("integer i; i := 3; outinteger(1, if i > 2 then 10 else 20); outinteger(1, 1 + (if i = 3 then 1 else 0))"), "10 2 ");
}

// ----- blocks, procedures, labels -----

#[test]
fn recursion() {
    let p = "integer procedure fact(n); value n; integer n; fact := if n = 0 then 1 else n × fact(n - 1);
             outinteger(1, fact(20))";
    assert_eq!(run(p), "2432902008176640000 ");
}

#[test]
fn an_inner_declaration_hides_an_outer_one() {
    assert_eq!(run("integer x; x := 1; begin real x; x := 2.5; outreal(1, x) end; outinteger(1, x)"), "2.5 1 ");
}

#[test]
fn array_bounds_are_evaluated_on_entry_to_the_block() {
    let p = "integer n; n := 3; begin integer array a[1:n]; n := 10; a[3] := 1; outinteger(1, a[3]) end";
    assert_eq!(run(p), "1 ");
    assert!(error("integer n; n := 3; begin integer array a[1:n]; a[4] := 1 end").contains("outside the bounds [1:3]"));
}

#[test]
fn an_array_with_an_upper_bound_below_its_lower_has_no_elements() {
    assert_eq!(run("integer n; n := 0; begin integer array a[1:n]; outstring(1, \"ok\") end"), "ok");
}

#[test]
fn own_variables_survive_between_activations() {
    let p = "integer procedure c; begin own integer n; n := n + 1; c := n end; c; c; outinteger(1, c)";
    assert_eq!(run(p), "3 ");
}

#[test]
fn a_label_parameter_jumps_to_its_own_activation() {
    let p = "procedure p(n, l); value n; integer n; label l;
             begin if n = 0 then go to l; p(n - 1, m); outinteger(1, -n); go to e; m: outinteger(1, n); e: end;
             p(2, done); done:";
    assert_eq!(run(p), "1 -2 ");
}

#[test]
fn an_undefined_switch_designator_makes_go_to_a_dummy() {
    assert_eq!(run("switch s := a; go to s[2]; outstring(1, \"fell through\"); a:"), "fell through");
}

#[test]
fn numeric_labels() {
    assert_eq!(run("go to 10; outstring(1, \"no\"); 0010: outstring(1, \"yes\")"), "yes");
}

#[test]
fn a_switch_element_is_evaluated_when_it_is_used() {
    let p = "integer i; switch s := if i = 1 then a else b; i := 1; go to s[1]; b: outstring(1, \"b\"); go to e; a: outstring(1, \"a\"); e:";
    assert_eq!(run(p), "a");
}

#[test]
fn a_procedure_body_need_not_be_a_block() {
    assert_eq!(run("integer procedure sq(x); value x; integer x; sq := x × x; outinteger(1, sq(12))"), "144 ");
}

#[test]
fn the_procedure_identifier_inside_its_body_is_a_recursive_call() {
    let p = "integer n; integer procedure f; begin n := n + 1; if n < 3 then f := f + 10 else f := 0 end; outinteger(1, f)";
    assert_eq!(run(p), "20 ");
}

// ----- the lexical rules -----

#[test]
fn text_after_end_is_a_comment() {
    assert_eq!(run("begin outstring(1, \"a\") end this is a comment; outstring(1, \"b\")"), "ab");
}

#[test]
fn a_missing_semicolon_after_end_swallows_the_next_statement() {
    let src = "begin integer i; begin i := 1 end\n i := 2; outinteger(1, i) end";
    let (out, r) = algol60::run_source(src, "", None);
    assert!(r.is_ok());
    assert_eq!(out, "1 ");
    assert!(algol60::check(src, algol60::Dialect::Report).unwrap()[0].contains("Is a ';' missing?"));
}

#[test]
fn comments() {
    assert_eq!(run("comment this is ignored; outstring(1, \"x\"); comment so is this;"), "x");
}

#[test]
fn strings_nest_in_the_reports_quotes() {
    assert_eq!(run("outstring(1, `a `b' c')"), "a `b' c");
}

#[test]
fn quote_stropping() {
    let src = "'BEGIN' 'INTEGER' A B; A B := 3; 'IF' AB 'GE' 3 'THEN' OUTINTEGER(1, A B) 'END'";
    assert_eq!(algol60::run_source(src, "", None).0, "3 ");
}

#[test]
fn numbers_with_exponents() {
    assert_eq!(run("outreal(1, 1.5⏨3); outreal(1, ⏨-2); outreal(1, .5); outreal(1, 2e1)"), "1500 0.01 0.5 20 ");
}

#[test]
fn parameter_delimiters_may_be_letter_strings() {
    let p = "procedure add(a, b) Result: (c); value a, b; integer a, b, c; c := a + b;
             integer r; add(2) plus: (3) Result: (r); outinteger(1, r)";
    assert_eq!(run(p), "5 ");
}

// ----- what the syntax refuses -----

fn refused(body: &str, why: &str) {
    let e = algol60::check(&format!("begin {} end", body), algol60::Dialect::Report).err().unwrap_or_else(|| panic!("accepted: {}", body));
    assert!(e.contains(why), "{} gave: {}", body, e);
}

#[test]
fn then_if_is_refused() {
    refused("integer i; if i = 1 then if i = 2 then i := 3", "must be enclosed in begin ... end");
}

#[test]
fn a_sign_after_an_operator_is_refused() {
    refused("integer i; i := 2 × -3", "a sign may only begin an expression");
}

#[test]
fn chained_relations_are_refused() {
    refused("Boolean b; b := 1 < 2 < 3", "a relation has one relational operator");
}

#[test]
fn else_after_a_for_statement_after_then_is_refused() {
    refused("integer i; if true then for i := 1 do i := 1 else i := 2", "cannot be followed by 'else'");
}

#[test]
fn undeclared_identifiers_are_refused() {
    refused("x := 1", "x is not declared");
}

#[test]
fn a_procedure_is_given_the_right_number_of_parameters() {
    refused("procedure p(a); a := 1; p(1, 2)", "p has 1 parameters, but is given 2");
}

#[test]
fn assignment_to_a_procedure_identifier_outside_its_body_is_refused() {
    refused("integer procedure f; f := 1; f := 2", "can only be assigned to inside the body of f");
}
