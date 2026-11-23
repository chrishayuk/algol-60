//! From source text to basic symbols (Revised Report, section 2).
//!
//! The reference language prints the word delimiters in bold or underlined.
//! Two hardware representations are read here:
//!
//! - **reserved words** (the default): `begin`, `real`, `if` and the rest
//!   are reserved, in any case, and spaces separate identifiers;
//! - **quote stropping**, chosen when the text contains `'begin'`: the word
//!   delimiters are written `'begin'`, `'REAL'`, and spaces are ignored
//!   everywhere outside strings, as the Report says (2.3), so `x 1` is `x1`;
//! - **underlining**, the reference language's own, chosen when the text
//!   contains U+0332 COMBINING LOW LINE: `b̲e̲g̲i̲n̲`. Spaces are ignored
//!   outside strings, as with quotes.
//!
//! Comments are removed here: `comment ... ;` after `begin` or `;`, and the
//! text after `end` up to the next `end`, `;` or `else` (2.3).

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(i64),
    Real(f64),
    Str(String),
    Begin,
    End,
    If,
    Then,
    Else,
    For,
    Do,
    Step,
    Until,
    While,
    Goto,
    Value,
    Own,
    Integer,
    RealKw,
    Boolean,
    Array,
    Switch,
    Procedure,
    StringKw,
    Label,
    True,
    False,
    Assign,
    Colon,
    Semi,
    Comma,
    LParen,
    RParen,
    LBrack,
    RBrack,
    Plus,
    Minus,
    Times,
    Slash,
    IntDiv,
    Power,
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
    Ne,
    Not,
    And,
    Or,
    Impl,
    Eqv,
    Eof,
}

impl Tok {
    pub fn describe(&self) -> String {
        use Tok::*;
        match self {
            Ident(s) => format!("identifier {}", s),
            Int(n) => format!("number {}", n),
            Real(x) => format!("number {}", x),
            Str(_) => "a string".into(),
            Eof => "the end of the program".into(),
            t => format!("'{}'", t.symbol()),
        }
    }

    pub fn symbol(&self) -> &'static str {
        use Tok::*;
        match self {
            Begin => "begin",
            End => "end",
            If => "if",
            Then => "then",
            Else => "else",
            For => "for",
            Do => "do",
            Step => "step",
            Until => "until",
            While => "while",
            Goto => "go to",
            Value => "value",
            Own => "own",
            Integer => "integer",
            RealKw => "real",
            Boolean => "Boolean",
            Array => "array",
            Switch => "switch",
            Procedure => "procedure",
            StringKw => "string",
            Label => "label",
            True => "true",
            False => "false",
            Assign => ":=",
            Colon => ":",
            Semi => ";",
            Comma => ",",
            LParen => "(",
            RParen => ")",
            LBrack => "[",
            RBrack => "]",
            Plus => "+",
            Minus => "-",
            Times => "×",
            Slash => "/",
            IntDiv => "÷",
            Power => "↑",
            Lt => "<",
            Le => "≤",
            Eq => "=",
            Ge => "≥",
            Gt => ">",
            Ne => "≠",
            Not => "¬",
            And => "∧",
            Or => "∨",
            Impl => "⊃",
            Eqv => "≡",
            Ident(_) | Int(_) | Real(_) | Str(_) | Eof => "",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub col: u32,
}

pub struct Lexed {
    pub tokens: Vec<Token>,
    pub warnings: Vec<String>,
    pub stropped: bool,
}

fn keyword(w: &str) -> Option<Tok> {
    use Tok::*;
    Some(match w {
        "begin" => Begin,
        "end" => End,
        "if" => If,
        "then" => Then,
        "else" => Else,
        "for" => For,
        "do" => Do,
        "step" => Step,
        "until" => Until,
        "while" => While,
        "goto" => Goto,
        "value" => Value,
        "own" => Own,
        "integer" => Integer,
        "real" => RealKw,
        "boolean" => Boolean,
        "array" => Array,
        "switch" => Switch,
        "procedure" => Procedure,
        "string" => StringKw,
        "label" => Label,
        "true" => True,
        "false" => False,
        "and" => And,
        "or" => Or,
        "not" => Not,
        "impl" | "imp" => Impl,
        "eqv" | "equiv" => Eqv,
        "div" => IntDiv,
        _ => return None,
    })
}

/// Words that are only delimiters when stropped: `'le'`, `'power'` and so on.
fn stropped_only(w: &str) -> Option<Tok> {
    use Tok::*;
    Some(match w {
        "lt" | "less" => Lt,
        "le" | "notgreater" => Le,
        "eq" | "equal" => Eq,
        "ge" | "notless" => Ge,
        "gt" | "greater" => Gt,
        "ne" | "notequal" => Ne,
        "power" => Power,
        "go to" => Goto,
        _ => return None,
    })
}

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    col: u32,
    /// Spaces are ignored: quote stropping or underlining.
    stropped: bool,
    quoted: bool,
    toks: Vec<Token>,
    warnings: Vec<String>,
}

pub fn lex(src: &str) -> Result<Lexed, String> {
    let quoted = src.to_lowercase().contains("'begin'");
    let stropped = quoted || src.contains(UNDERLINE);
    let mut lx =
        Lexer { chars: src.chars().collect(), i: 0, line: 1, col: 1, stropped, quoted, toks: vec![], warnings: vec![] };
    lx.run()?;
    Ok(Lexed { tokens: lx.toks, warnings: lx.warnings, stropped })
}

const UNDERLINE: char = '\u{332}';

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek_at(&self, k: usize) -> Option<char> {
        self.chars.get(self.i + k).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn err(&self, line: u32, col: u32, msg: &str) -> String {
        format!("line {}, column {}: {}", line, col, msg)
    }

    fn push(&mut self, tok: Tok, line: u32, col: u32) {
        self.toks.push(Token { tok, line, col });
    }

    fn run(&mut self) -> Result<(), String> {
        loop {
            while self.peek().is_some_and(|c| c.is_whitespace()) {
                self.bump();
            }
            let (line, col) = (self.line, self.col);
            let Some(c) = self.peek() else {
                self.push(Tok::Eof, line, col);
                return Ok(());
            };
            if self.peek_at(1) == Some(UNDERLINE) {
                self.underlined_word(line, col)?;
            } else if c.is_alphabetic() {
                self.word(line, col)?;
            } else if c.is_ascii_digit()
                || (c == '.' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()))
                || c == '⏨'
                || c == '#'
            {
                let t = self.number(line, col)?;
                self.push(t, line, col);
            } else if c == '\'' && self.quoted {
                self.stropped_word(line, col)?;
            } else {
                self.symbol(line, col)?;
            }
        }
    }

    fn word(&mut self, line: u32, col: u32) -> Result<(), String> {
        let mut w = String::new();
        loop {
            match self.peek() {
                Some(c) if is_word_char(c) && self.peek_at(1) != Some(UNDERLINE) => {
                    w.push(c);
                    self.bump();
                }
                // With quote stropping, spaces inside an identifier are ignored.
                Some(c) if self.stropped && c.is_whitespace() => {
                    let mut k = 0;
                    while self.peek_at(k).is_some_and(|c| c.is_whitespace()) {
                        k += 1;
                    }
                    if self.peek_at(k).is_some_and(is_word_char) && self.peek_at(k + 1) != Some(UNDERLINE) {
                        for _ in 0..k {
                            self.bump();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        if self.stropped {
            self.push(Tok::Ident(w), line, col);
            return Ok(());
        }
        // A reserved word is written in lower case or in capitals (or as
        // `Boolean`), so that `Step`, a procedure in the Report's own
        // examples, is an identifier.
        let lw = if w == w.to_lowercase() || w == w.to_uppercase() || w == "Boolean" {
            w.to_lowercase()
        } else {
            String::new()
        };
        if lw == "comment" {
            return self.comment(line, col);
        }
        if lw == "go" {
            // "go to" is one basic symbol.
            let save = (self.i, self.line, self.col);
            while self.peek().is_some_and(|c| c.is_whitespace()) {
                self.bump();
            }
            let mut next = String::new();
            while let Some(c) = self.peek().filter(|&c| is_word_char(c)) {
                next.push(c);
                self.bump();
            }
            if next == "to" || next == "TO" {
                self.push(Tok::Goto, line, col);
                return Ok(());
            }
            (self.i, self.line, self.col) = save;
        }
        match keyword(&lw) {
            Some(Tok::End) => {
                self.push(Tok::End, line, col);
                self.end_comment();
            }
            Some(k) => self.push(k, line, col),
            None => self.push(Tok::Ident(w), line, col),
        }
        Ok(())
    }

    fn stropped_word(&mut self, line: u32, col: u32) -> Result<(), String> {
        self.bump();
        let mut w = String::new();
        loop {
            match self.bump() {
                Some('\'') => break,
                Some(c) if c.is_whitespace() => {}
                Some(c) => w.push(c),
                None => return Err(self.err(line, col, "a quoted word delimiter is not closed")),
            }
        }
        let lw = w.to_lowercase();
        if lw == "comment" {
            return self.comment(line, col);
        }
        if lw == "10" {
            let t = self.number_from_exponent(line, col)?;
            self.push(t, line, col);
            return Ok(());
        }
        let tok = if lw == "goto" || lw == "go" {
            // 'go' 'to' as two quoted words
            if lw == "go" {
                let save = (self.i, self.line, self.col);
                while self.peek().is_some_and(|c| c.is_whitespace()) {
                    self.bump();
                }
                let mut rest = String::new();
                if self.peek() == Some('\'') {
                    self.bump();
                    while let Some(c) = self.bump() {
                        if c == '\'' {
                            break;
                        }
                        rest.push(c);
                    }
                }
                if rest.to_lowercase().trim() != "to" {
                    (self.i, self.line, self.col) = save;
                    return Err(self.err(line, col, "'go' is not a word delimiter (did you mean 'go to'?)"));
                }
            }
            Tok::Goto
        } else if let Some(t) = keyword(&lw).or_else(|| stropped_only(&lw)) {
            t
        } else {
            return Err(self.err(line, col, &format!("'{}' is not a word delimiter", w)));
        };
        let is_end = tok == Tok::End;
        self.push(tok, line, col);
        if is_end {
            self.end_comment();
        }
        Ok(())
    }

    /// The underlined word starting k characters on, in lower case, and
    /// how many characters it takes. `g̲o̲ t̲o̲` is one word.
    fn underlined_at(&self, mut k: usize) -> (String, usize) {
        let start = k;
        let mut w = String::new();
        loop {
            match (self.peek_at(k), self.peek_at(k + 1)) {
                (Some(c), Some(UNDERLINE)) if !c.is_whitespace() => {
                    w.extend(c.to_lowercase());
                    k += 2;
                    // A doubled underline is still one.
                    while self.peek_at(k) == Some(UNDERLINE) {
                        k += 1;
                    }
                }
                (Some(c), _) if c.is_whitespace() && w == "go" => {
                    let mut j = k;
                    while self.peek_at(j).is_some_and(|c| c.is_whitespace()) {
                        j += 1;
                    }
                    if self.peek_at(j + 1) == Some(UNDERLINE) {
                        k = j;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        (w, k - start)
    }

    fn underlined_word(&mut self, line: u32, col: u32) -> Result<(), String> {
        let (w, n) = self.underlined_at(0);
        for _ in 0..n {
            self.bump();
        }
        if w == "comment" {
            return self.comment(line, col);
        }
        let tok = match w.as_str() {
            "goto" | "go" => Tok::Goto,
            _ => match keyword(&w).or_else(|| stropped_only(&w)) {
                Some(t) => t,
                None => return Err(self.err(line, col, &format!("{} is not a word delimiter", w))),
            },
        };
        let is_end = tok == Tok::End;
        self.push(tok, line, col);
        if is_end {
            self.end_comment();
        }
        Ok(())
    }

    /// `comment` up to and including the next `;`.
    fn comment(&mut self, line: u32, col: u32) -> Result<(), String> {
        loop {
            match self.bump() {
                Some(';') => return Ok(()),
                Some(_) => {}
                None => return Err(self.err(line, col, "a comment is not ended by ';'")),
            }
        }
    }

    /// After `end`, everything up to the next `end`, `;` or `else` is a
    /// comment (2.3). A missing semicolon after `end` therefore makes the
    /// next statement disappear; that is warned about.
    fn end_comment(&mut self) {
        let start_line = self.line;
        let mut text = String::new();
        loop {
            let Some(c) = self.peek() else { break };
            if c == ';' {
                break;
            }
            if self.peek_at(1) == Some(UNDERLINE) {
                let w = self.underlined_at(0).0;
                if w == "end" || w == "else" {
                    break;
                }
            } else if self.quoted {
                if c == '\'' {
                    let mut k = 1;
                    let mut w = String::new();
                    while let Some(d) = self.peek_at(k) {
                        if d == '\'' {
                            break;
                        }
                        if !d.is_whitespace() {
                            w.push(d);
                        }
                        k += 1;
                    }
                    let lw = w.to_lowercase();
                    if lw == "end" || lw == "else" {
                        break;
                    }
                }
            } else if c.is_alphabetic() && !self.i.checked_sub(1).and_then(|j| self.chars.get(j)).is_some_and(|&p| is_word_char(p)) {
                let mut k = 0;
                let mut w = String::new();
                while let Some(d) = self.peek_at(k).filter(|&d| is_word_char(d)) {
                    w.push(d);
                    k += 1;
                }
                if matches!(w.as_str(), "end" | "END" | "else" | "ELSE") {
                    break;
                }
            }
            text.push(c);
            self.bump();
        }
        if text.contains(":=") {
            let shown: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
            let shown: String = shown.chars().take(40).collect();
            self.warnings.push(format!(
                "line {}: the text after 'end' up to the next ';', 'end' or 'else' is a comment, and it contains ':=' ({}). Is a ';' missing?",
                start_line, shown
            ));
        }
    }

    /// Digits, and when spaces are not significant, digits after spaces
    /// too: `.27639 32022 50021` is one number.
    fn digits(&mut self) -> String {
        let mut s = String::new();
        loop {
            while let Some(c) = self.peek().filter(|c| c.is_ascii_digit()) {
                s.push(c);
                self.bump();
            }
            if !self.stropped || s.is_empty() {
                return s;
            }
            let mut k = 0;
            while self.peek_at(k).is_some_and(|c| c == ' ' || c == '\t') {
                k += 1;
            }
            if k == 0 || !self.peek_at(k).is_some_and(|c| c.is_ascii_digit()) || self.peek_at(k + 1) == Some(UNDERLINE) {
                return s;
            }
            for _ in 0..k {
                self.bump();
            }
        }
    }

    fn number(&mut self, line: u32, col: u32) -> Result<Tok, String> {
        let int_part = self.digits();
        let mut frac = None;
        if self.peek() == Some('.') && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()) {
            self.bump();
            frac = Some(self.digits());
        }
        let exp_mark = match self.peek() {
            Some('⏨') | Some('#') => true,
            Some('e') | Some('E') if !self.stropped => {
                let next = self.peek_at(1);
                next.is_some_and(|d| d.is_ascii_digit())
                    || (matches!(next, Some('+') | Some('-')) && self.peek_at(2).is_some_and(|d| d.is_ascii_digit()))
            }
            Some('\'') if self.quoted => {
                // '10' as the exponent marker
                let w: String = (1..4).filter_map(|k| self.peek_at(k)).collect();
                w == "10'"
            }
            _ => false,
        };
        if exp_mark && self.peek() == Some('\'') {
            for _ in 0..4 {
                self.bump();
            }
        } else if exp_mark {
            self.bump();
        }
        if int_part.is_empty() && frac.is_none() && !exp_mark {
            return Err(self.err(line, col, "a number expected"));
        }
        if !exp_mark && frac.is_none() {
            return int_part.parse::<i64>().map(Tok::Int).map_err(|_| self.err(line, col, "integer too large"));
        }
        let mantissa: f64 = if int_part.is_empty() && frac.is_none() {
            1.0
        } else {
            format!("{}.{}", if int_part.is_empty() { "0" } else { &int_part }, frac.as_deref().unwrap_or("0"))
                .parse()
                .unwrap()
        };
        if exp_mark {
            let neg = match self.peek() {
                Some('+') => {
                    self.bump();
                    false
                }
                Some('-') | Some('−') => {
                    self.bump();
                    true
                }
                _ => false,
            };
            return self.exponent_digits(line, col, mantissa, neg);
        }
        Ok(Tok::Real(mantissa))
    }

    fn number_from_exponent(&mut self, line: u32, col: u32) -> Result<Tok, String> {
        let neg = match self.peek() {
            Some('+') => {
                self.bump();
                false
            }
            Some('-') => {
                self.bump();
                true
            }
            _ => false,
        };
        self.exponent_digits(line, col, 1.0, neg)
    }

    fn exponent_digits(&mut self, line: u32, col: u32, mantissa: f64, neg: bool) -> Result<Tok, String> {
        let e = self.digits();
        if e.is_empty() {
            return Err(self.err(line, col, "digits expected after ⏨"));
        }
        let x: f64 = format!("{}e{}{}", mantissa, if neg { "-" } else { "" }, e).parse().unwrap();
        if !x.is_finite() {
            return Err(self.err(line, col, "number too large"));
        }
        Ok(Tok::Real(x))
    }

    fn string(&mut self, line: u32, col: u32, open: char) -> Result<Tok, String> {
        self.bump();
        let mut s = String::new();
        if open == '"' {
            loop {
                match self.bump() {
                    Some('"') => break,
                    Some('\\') => match self.bump() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some(c) => s.push(c),
                        None => break,
                    },
                    Some(c) => s.push(c),
                    None => return Err(self.err(line, col, "a string is not closed")),
                }
            }
            return Ok(Tok::Str(s));
        }
        // The Report's own strings nest: `a `b' c' is one string.
        let close = match open {
            '`' => '\'',
            '{' => '}',
            _ => '’',
        };
        let mut depth = 1;
        loop {
            match self.bump() {
                Some(c) if c == open => {
                    depth += 1;
                    s.push(c);
                }
                Some(c) if c == close => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    s.push(c);
                }
                Some(c) => s.push(c),
                None => return Err(self.err(line, col, "a string is not closed")),
            }
        }
        Ok(Tok::Str(s))
    }

    fn symbol(&mut self, line: u32, col: u32) -> Result<(), String> {
        use Tok::*;
        let c = self.peek().unwrap();
        // `{...}` is how some transcriptions write the Report's quotes.
        if c == '"' || c == '`' || c == '‘' || c == '{' {
            let t = self.string(line, col, c)?;
            self.push(t, line, col);
            return Ok(());
        }
        self.bump();
        let next = self.peek();
        let two = |lx: &mut Lexer, t: Tok| {
            lx.bump();
            t
        };
        let t = match c {
            ':' if next == Some('=') => two(self, Assign),
            ':' => Colon,
            ';' => Semi,
            ',' => Comma,
            '(' => LParen,
            ')' => RParen,
            '[' => LBrack,
            ']' => RBrack,
            '+' => Plus,
            '-' | '−' => Minus,
            '*' if next == Some('*') => two(self, Power),
            '*' | '×' => Times,
            '/' => Slash,
            '÷' => IntDiv,
            '↑' | '^' => Power,
            '<' if next == Some('=') => two(self, Le),
            '<' if next == Some('>') => two(self, Ne),
            '<' => Lt,
            '>' if next == Some('=') => two(self, Ge),
            '>' => Gt,
            '=' => Eq,
            '≠' => Ne,
            '≤' => Le,
            '≥' => Ge,
            '¬' | '~' => Not,
            '∧' => And,
            '∨' => Or,
            '⊃' => Impl,
            '≡' => Eqv,
            _ => return Err(self.err(line, col, &format!("'{}' is not a basic symbol", c))),
        };
        self.push(t, line, col);
        Ok(())
    }
}
