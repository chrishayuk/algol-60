//! Recursive descent over the syntax of the Revised Report, sections 3 to 5.
//!
//! Where the Report's syntax refuses something, so does this parser, with a
//! message saying what the Report wants instead: `if a then if b ...`, a
//! sign after an operator (`a × -b`), `a < b < c`, and an `else` after a
//! for statement that follows `then`.

use crate::ast::*;
use crate::lexer::{Tok, Token};

type PResult<T> = Result<T, String>;

pub struct Parser {
    toks: Vec<Token>,
    i: usize,
    /// Accept `then if`, as the X1 compiler did.
    x1: bool,
    notes: Notes,
}

/// What the parser saw that the X1's manual forbade.
#[derive(Default)]
pub struct Notes {
    /// Lines where a parameter delimiter is a letter string.
    pub letter_delimiters: Vec<u32>,
    /// Lines where `if` follows `then` (accepted only for the X1).
    pub then_ifs: Vec<u32>,
}

pub fn parse_program(toks: Vec<Token>, x1: bool) -> PResult<(Stmt, Notes)> {
    let mut p = Parser { toks, i: 0, x1, notes: Notes::default() };
    let s = p.statement()?;
    while p.eat(&Tok::Semi) {}
    if *p.peek() != Tok::Eof {
        return Err(p.error("the end of the program expected"));
    }
    match s.kind {
        SK::Block(_) | SK::Compound(_) => Ok((s, p.notes)),
        _ => Err("line 1: a program is a block or a compound statement (begin ... end)".into()),
    }
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.i].tok
    }

    fn peek_at(&self, k: usize) -> &Tok {
        let j = (self.i + k).min(self.toks.len() - 1);
        &self.toks[j].tok
    }

    fn line(&self) -> u32 {
        self.toks[self.i].line
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.i].tok.clone();
        if self.i < self.toks.len() - 1 {
            self.i += 1;
        }
        t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == t {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: Tok) -> PResult<()> {
        if self.eat(&t) {
            Ok(())
        } else {
            Err(self.error(&format!("'{}' expected", t.symbol())))
        }
    }

    fn error(&self, msg: &str) -> String {
        let t = &self.toks[self.i];
        format!("line {}, column {}: {} (found {})", t.line, t.col, msg, t.tok.describe())
    }

    fn ident(&mut self) -> PResult<Name> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Ident(n) => {
                self.bump();
                Ok((n, line))
            }
            _ => Err(self.error("an identifier expected")),
        }
    }

    // ----- statements (section 4) -----

    fn statement(&mut self) -> PResult<Stmt> {
        let line = self.line();
        let mut labels = Vec::new();
        loop {
            let a = self.peek().clone();
            let b = self.peek_at(1).clone();
            let l = self.line();
            match (a, b) {
                (Tok::Ident(n), Tok::Colon) => labels.push((n, l)),
                // Unsigned integers are labels too (3.5.1); leading zeros do
                // not matter.
                (Tok::Int(n), Tok::Colon) => labels.push((n.to_string(), l)),
                _ => break,
            }
            self.bump();
            self.bump();
        }
        let kind = self.unlabelled()?;
        Ok(Stmt { labels, kind, line })
    }

    fn unlabelled(&mut self) -> PResult<SK> {
        match self.peek() {
            Tok::Begin => {
                self.bump();
                self.block_or_compound()
            }
            Tok::If => self.conditional(),
            Tok::For => self.for_statement(),
            Tok::Goto => {
                self.bump();
                Ok(SK::Goto(self.expr()?))
            }
            Tok::Ident(_) => self.assign_or_call(),
            Tok::Semi | Tok::End | Tok::Else | Tok::Eof => Ok(SK::Dummy),
            _ => Err(self.error("a statement expected")),
        }
    }

    fn is_decl_start(&self) -> bool {
        matches!(
            self.peek(),
            Tok::Own | Tok::Integer | Tok::RealKw | Tok::Boolean | Tok::Array | Tok::Switch | Tok::Procedure
        )
    }

    fn block_or_compound(&mut self) -> PResult<SK> {
        let mut decls = Vec::new();
        while self.is_decl_start() {
            decls.push(self.declaration()?);
            if !self.eat(&Tok::Semi) {
                return Err(self.error("';' expected after a declaration"));
            }
        }
        let mut body = vec![self.statement()?];
        loop {
            if self.eat(&Tok::Semi) {
                if self.is_decl_start() {
                    return Err(self.error("declarations must come at the start of a block, before any statement"));
                }
                body.push(self.statement()?);
            } else if self.eat(&Tok::End) {
                break;
            } else {
                return Err(self.error("';' or 'end' expected"));
            }
        }
        Ok(if decls.is_empty() { SK::Compound(body) } else { SK::Block(Block { decls, body }) })
    }

    fn conditional(&mut self) -> PResult<SK> {
        self.bump();
        let cond = self.expr()?;
        self.expect(Tok::Then)?;
        if *self.peek() == Tok::If {
            if !self.x1 {
                return Err(self.error(
                    "after 'then' a conditional statement must be enclosed in begin ... end (4.5.1)",
                ));
            }
            // The X1 compiler, which checked almost no syntax, took the
            // inner conditional statement whole, so that an `else` goes
            // with the nearest `if`.
            self.notes.then_ifs.push(self.line());
            let inner = self.statement()?;
            return Ok(SK::If(cond, Box::new(inner), None));
        }
        let then = self.statement()?;
        if matches!(then.kind, SK::If(..)) {
            return Err(format!(
                "line {}: after 'then' a labelled conditional statement must be enclosed in begin ... end (4.5.1)",
                then.line
            ));
        }
        if self.eat(&Tok::Else) {
            if matches!(then.kind, SK::For(..)) {
                return Err(format!(
                    "line {}: a for statement after 'then' cannot be followed by 'else'; enclose it in begin ... end (4.5.1)",
                    then.line
                ));
            }
            let other = self.statement()?;
            Ok(SK::If(cond, Box::new(then), Some(Box::new(other))))
        } else {
            Ok(SK::If(cond, Box::new(then), None))
        }
    }

    fn for_statement(&mut self) -> PResult<SK> {
        self.bump();
        let var = self.variable()?;
        self.expect(Tok::Assign)?;
        let mut elems = Vec::new();
        loop {
            let e = self.expr()?;
            if self.eat(&Tok::Step) {
                let b = self.expr()?;
                self.expect(Tok::Until)?;
                let c = self.expr()?;
                elems.push(ForElem::Step(e, b, c));
            } else if self.eat(&Tok::While) {
                elems.push(ForElem::While(e, self.expr()?));
            } else {
                elems.push(ForElem::Expr(e));
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(Tok::Do)?;
        let body = self.statement()?;
        Ok(SK::For(var, elems, Box::new(body)))
    }

    /// A simple or subscripted variable.
    fn variable(&mut self) -> PResult<E> {
        let (name, line) = self.ident()?;
        if self.eat(&Tok::LBrack) {
            let subs = self.expr_list()?;
            self.expect(Tok::RBrack)?;
            Ok(E::Sub(name, subs, line))
        } else {
            Ok(E::Id(name, line))
        }
    }

    fn expr_list(&mut self) -> PResult<Vec<E>> {
        let mut v = vec![self.expr()?];
        while self.eat(&Tok::Comma) {
            v.push(self.expr()?);
        }
        Ok(v)
    }

    fn assign_or_call(&mut self) -> PResult<SK> {
        let start = self.i;
        let mut lefts = Vec::new();
        loop {
            let s = self.i;
            if !matches!(self.peek(), Tok::Ident(_)) {
                break;
            }
            let v = self.variable()?;
            if self.eat(&Tok::Assign) {
                lefts.push(v);
            } else {
                self.i = s;
                break;
            }
        }
        if !lefts.is_empty() {
            let e = self.expr()?;
            return Ok(SK::Assign(lefts, e));
        }
        self.i = start;
        let (name, line) = self.ident()?;
        let args = if self.eat(&Tok::LParen) { self.actuals()? } else { Vec::new() };
        if *self.peek() == Tok::LBrack {
            return Err(self.error("':=' expected"));
        }
        Ok(SK::Call(name, args, line))
    }

    /// The comma between parameters may be written `) letter string :(`,
    /// as in `Spur (A) Order: (7) Result to: (V)` (4.7.1).
    fn param_delimiter(&mut self) -> bool {
        if self.eat(&Tok::Comma) {
            return true;
        }
        if *self.peek() == Tok::RParen {
            let mut k = 1;
            while matches!(self.peek_at(k), Tok::Ident(_)) {
                k += 1;
            }
            if k > 1 && *self.peek_at(k) == Tok::Colon && *self.peek_at(k + 1) == Tok::LParen {
                self.notes.letter_delimiters.push(self.line());
                for _ in 0..k + 2 {
                    self.bump();
                }
                return true;
            }
        }
        false
    }

    fn actuals(&mut self) -> PResult<Vec<E>> {
        let mut v = Vec::new();
        loop {
            v.push(self.expr()?);
            if !self.param_delimiter() {
                break;
            }
        }
        self.expect(Tok::RParen)?;
        Ok(v)
    }

    // ----- declarations (section 5) -----

    fn names(&mut self) -> PResult<Vec<Name>> {
        let mut v = vec![self.ident()?];
        while self.eat(&Tok::Comma) {
            v.push(self.ident()?);
        }
        Ok(v)
    }

    fn declaration(&mut self) -> PResult<Decl> {
        let own = self.eat(&Tok::Own);
        let ty = match self.peek() {
            Tok::Integer => Some(Type::Integer),
            Tok::RealKw => Some(Type::Real),
            Tok::Boolean => Some(Type::Boolean),
            _ => None,
        };
        if ty.is_some() {
            self.bump();
        }
        match self.peek() {
            Tok::Array => {
                self.bump();
                let mut segs = Vec::new();
                loop {
                    let names = self.names()?;
                    self.expect(Tok::LBrack)?;
                    let mut bounds = Vec::new();
                    loop {
                        let lo = self.expr()?;
                        self.expect(Tok::Colon)?;
                        let hi = self.expr()?;
                        bounds.push((lo, hi));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(Tok::RBrack)?;
                    segs.push((names, bounds));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                Ok(Decl::Array { own, ty: ty.unwrap_or(Type::Real), segs })
            }
            Tok::Procedure if !own => {
                self.bump();
                self.procedure(ty)
            }
            Tok::Switch if !own && ty.is_none() => {
                self.bump();
                let name = self.ident()?;
                self.expect(Tok::Assign)?;
                let elems = self.expr_list()?;
                Ok(Decl::Switch { name, elems })
            }
            _ if ty.is_some() => Ok(Decl::Simple { own, ty: ty.unwrap(), names: self.names()? }),
            _ => Err(self.error("a declaration expected")),
        }
    }

    fn procedure(&mut self, ty: Option<Type>) -> PResult<Decl> {
        let name = self.ident()?;
        let mut params = Vec::new();
        if self.eat(&Tok::LParen) {
            loop {
                params.push(self.ident()?);
                if !self.param_delimiter() {
                    break;
                }
            }
            self.expect(Tok::RParen)?;
        }
        self.expect(Tok::Semi)?;
        let mut values = Vec::new();
        let mut specs = Vec::new();
        loop {
            if self.eat(&Tok::Value) {
                values.extend(self.names()?);
                self.expect(Tok::Semi)?;
                continue;
            }
            let spec = match self.peek() {
                Tok::StringKw => Spec::Str,
                Tok::Label => Spec::Label,
                Tok::Switch => Spec::Switch,
                Tok::Array => Spec::Array(Type::Real),
                Tok::Procedure => Spec::Proc(None),
                Tok::Integer | Tok::RealKw | Tok::Boolean => {
                    let t = match self.bump() {
                        Tok::Integer => Type::Integer,
                        Tok::RealKw => Type::Real,
                        _ => Type::Boolean,
                    };
                    let spec = match self.peek() {
                        Tok::Array => Spec::Array(t),
                        Tok::Procedure => Spec::Proc(Some(t)),
                        _ => Spec::Simple(t),
                    };
                    if !matches!(spec, Spec::Simple(_)) {
                        self.bump();
                    }
                    let names = self.names()?;
                    self.expect(Tok::Semi)?;
                    specs.push((spec, names));
                    continue;
                }
                _ => break,
            };
            self.bump();
            let names = self.names()?;
            self.expect(Tok::Semi)?;
            specs.push((spec, names));
        }
        let body = self.statement()?;
        Ok(Decl::Proc(ProcDecl { name, ty, params, values, specs, body: Box::new(body) }))
    }

    // ----- expressions (section 3) -----

    pub fn expr(&mut self) -> PResult<E> {
        if self.eat(&Tok::If) {
            let c = self.expr()?;
            self.expect(Tok::Then)?;
            if *self.peek() == Tok::If {
                return Err(self.error("after 'then' a conditional expression must be in parentheses (3.3.1)"));
            }
            let a = self.simple()?;
            self.expect(Tok::Else)?;
            let b = self.expr()?;
            return Ok(E::If(Box::new(c), Box::new(a), Box::new(b)));
        }
        self.simple()
    }

    fn simple(&mut self) -> PResult<E> {
        let mut l = self.implication()?;
        while self.eat(&Tok::Eqv) {
            let r = self.implication()?;
            l = E::Bin(BOp::Eqv, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn implication(&mut self) -> PResult<E> {
        let mut l = self.bool_term()?;
        while self.eat(&Tok::Impl) {
            let r = self.bool_term()?;
            l = E::Bin(BOp::Impl, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn bool_term(&mut self) -> PResult<E> {
        let mut l = self.bool_factor()?;
        while self.eat(&Tok::Or) {
            let r = self.bool_factor()?;
            l = E::Bin(BOp::Or, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn bool_factor(&mut self) -> PResult<E> {
        let mut l = self.bool_secondary()?;
        while self.eat(&Tok::And) {
            let r = self.bool_secondary()?;
            l = E::Bin(BOp::And, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn bool_secondary(&mut self) -> PResult<E> {
        if self.eat(&Tok::Not) {
            return Ok(E::Not(Box::new(self.bool_secondary()?)));
        }
        self.relation()
    }

    fn relop(&self) -> Option<BOp> {
        Some(match self.peek() {
            Tok::Lt => BOp::Lt,
            Tok::Le => BOp::Le,
            Tok::Eq => BOp::Eq,
            Tok::Ge => BOp::Ge,
            Tok::Gt => BOp::Gt,
            Tok::Ne => BOp::Ne,
            _ => return None,
        })
    }

    fn relation(&mut self) -> PResult<E> {
        let a = self.arith()?;
        if let Some(op) = self.relop() {
            self.bump();
            let b = self.arith()?;
            if self.relop().is_some() {
                return Err(self.error("a relation has one relational operator; write (a < b) ∧ (b < c) (3.4.1)"));
            }
            return Ok(E::Bin(op, Box::new(a), Box::new(b)));
        }
        Ok(a)
    }

    fn arith(&mut self) -> PResult<E> {
        let mut l = if self.eat(&Tok::Plus) {
            E::Pos(Box::new(self.term()?))
        } else if self.eat(&Tok::Minus) {
            E::Neg(Box::new(self.term()?))
        } else {
            self.term()?
        };
        loop {
            let op = match self.peek() {
                Tok::Plus => BOp::Add,
                Tok::Minus => BOp::Sub,
                _ => break,
            };
            self.bump();
            let r = self.term()?;
            l = E::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn term(&mut self) -> PResult<E> {
        let mut l = self.factor()?;
        loop {
            let op = match self.peek() {
                Tok::Times => BOp::Mul,
                Tok::Slash => BOp::Div,
                Tok::IntDiv => BOp::IntDiv,
                _ => break,
            };
            self.bump();
            let r = self.factor()?;
            l = E::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    /// `↑` groups from the left: 2↑3↑2 is (2↑3)↑2 = 64 (3.3.5.2).
    fn factor(&mut self) -> PResult<E> {
        let mut l = self.primary()?;
        while self.eat(&Tok::Power) {
            let r = self.primary()?;
            l = E::Bin(BOp::Pow, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn primary(&mut self) -> PResult<E> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Int(n) => {
                self.bump();
                Ok(E::Int(n))
            }
            Tok::Real(x) => {
                self.bump();
                Ok(E::Real(x))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(E::Str(s))
            }
            Tok::True => {
                self.bump();
                Ok(E::Bool(true))
            }
            Tok::False => {
                self.bump();
                Ok(E::Bool(false))
            }
            Tok::LParen => {
                self.bump();
                let e = self.expr()?;
                self.expect(Tok::RParen)?;
                Ok(e)
            }
            Tok::Ident(name) => {
                self.bump();
                if self.eat(&Tok::LBrack) {
                    let subs = self.expr_list()?;
                    self.expect(Tok::RBrack)?;
                    Ok(E::Sub(name, subs, line))
                } else if self.eat(&Tok::LParen) {
                    Ok(E::Call(name, self.actuals()?, line))
                } else {
                    Ok(E::Id(name, line))
                }
            }
            Tok::Plus | Tok::Minus => Err(self.error("a sign may only begin an expression; use parentheses, as in a × (-b) (3.3.1)")),
            Tok::If => Err(self.error("a conditional expression here must be in parentheses (3.3.1)")),
            _ => Err(self.error("an expression expected")),
        }
    }
}
