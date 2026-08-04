//! Parse free-text catalog `prereq` fields into course-code boolean expressions
//! and detect missing prerequisites for courses on a plan (warn-only).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::course;
use crate::course_relations;
use crate::penn_data::courses_data;

static PREREQ_MAP: OnceLock<HashMap<String, Expr>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Code(String),
    And(Vec<Expr>),
    Or(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Code(String),
    And,
    Or,
    LParen,
    RParen,
}

fn prereq_map() -> &'static HashMap<String, Expr> {
    PREREQ_MAP.get_or_init(build_prereq_map)
}

fn build_prereq_map() -> HashMap<String, Expr> {
    let mut map = HashMap::new();
    for c in courses_data::courses() {
        let code = course_relations::normalize_code(&c.course_code);
        if !course::is_valid_course_code(&code) {
            continue;
        }
        let Some(raw) = c.prereq.as_deref() else {
            continue;
        };
        if let Some(expr) = parse_prereq(raw) {
            map.insert(code, expr);
        }
    }
    map
}

/// Parse a raw catalog prereq string (tests).
fn parse_prereq(raw: &str) -> Option<Expr> {
    let tokens = tokenize(raw);
    if tokens.is_empty() {
        return None;
    }
    let tokens = insert_implicit_ands(tokens);
    let mut i = 0;
    let expr = parse_or(&tokens, &mut i)?;
    if i != tokens.len() {
        // Trailing junk is fine if we got a usable expression.
    }
    Some(flatten(expr))
}

fn tokenize(raw: &str) -> Vec<Token> {
    let chars: Vec<char> = raw
        .chars()
        .map(|c| if c == '\u{00a0}' { ' ' } else { c })
        .collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        if chars[i] == '(' {
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }
        if chars[i] == ')' {
            tokens.push(Token::RParen);
            i += 1;
            continue;
        }
        if let Some(ni) = match_keyword(&chars, i, "AND") {
            tokens.push(Token::And);
            i = ni;
            continue;
        }
        if let Some(ni) = match_keyword(&chars, i, "OR") {
            tokens.push(Token::Or);
            i = ni;
            continue;
        }
        if let Some((code, ni)) = match_course_code(&chars, i) {
            tokens.push(Token::Code(code));
            i = ni;
            continue;
        }
        // Skip prose / punctuation.
        i += 1;
    }
    tokens
}

fn match_keyword(chars: &[char], i: usize, word: &str) -> Option<usize> {
    let w: Vec<char> = word.chars().collect();
    if i + w.len() > chars.len() {
        return None;
    }
    for (j, wc) in w.iter().enumerate() {
        if !chars[i + j].eq_ignore_ascii_case(wc) {
            return None;
        }
    }
    let end = i + w.len();
    // Require a non-letter before the keyword so we don't match inside words
    // like "STANDARD". Digits/parens are allowed so glued forms work:
    // `CIS 1200ANDCIS 1600`, `ACCT 6110ORACCT 6130`.
    if i > 0 {
        let p = chars[i - 1];
        if p.is_ascii_alphabetic() {
            return None;
        }
    }
    Some(end)
}

fn match_course_code(chars: &[char], i: usize) -> Option<(String, usize)> {
    if !chars[i].is_ascii_alphabetic() {
        return None;
    }
    let mut j = i;
    while j < chars.len() && chars[j].is_ascii_alphabetic() {
        j += 1;
    }
    let dept: String = chars[i..j].iter().collect::<String>().to_uppercase();
    if dept.len() < 2 || dept.len() > 5 {
        return None;
    }
    while j < chars.len() && chars[j].is_whitespace() {
        j += 1;
    }
    if j >= chars.len() || !chars[j].is_ascii_digit() {
        return None;
    }
    let num_start = j;
    while j < chars.len() && chars[j].is_ascii_digit() {
        j += 1;
    }
    let num: String = chars[num_start..j].iter().collect();
    if num.len() < 3 || num.len() > 4 {
        return None;
    }
    let code = format!("{dept} {num}");
    if !course::is_valid_course_code(&code) {
        return None;
    }
    Some((code, j))
}

/// `A B` with no connector → treat as `A AND B` (common after prose stripping).
fn insert_implicit_ands(tokens: Vec<Token>) -> Vec<Token> {
    let mut out = Vec::new();
    for tok in tokens {
        if let (Some(Token::Code(_)), Token::Code(_)) = (out.last(), &tok) {
            out.push(Token::And);
        }
        if let (Some(Token::RParen), Token::Code(_)) = (out.last(), &tok) {
            out.push(Token::And);
        }
        if let (Some(Token::Code(_)), Token::LParen) = (out.last(), &tok) {
            out.push(Token::And);
        }
        out.push(tok);
    }
    out
}

fn parse_or(tokens: &[Token], i: &mut usize) -> Option<Expr> {
    let mut parts = vec![parse_and(tokens, i)?];
    while *i < tokens.len() {
        if tokens[*i] != Token::Or {
            break;
        }
        *i += 1;
        parts.push(parse_and(tokens, i)?);
    }
    Some(if parts.len() == 1 {
        parts.pop().unwrap()
    } else {
        Expr::Or(parts)
    })
}

fn parse_and(tokens: &[Token], i: &mut usize) -> Option<Expr> {
    let mut parts = vec![parse_factor(tokens, i)?];
    while *i < tokens.len() {
        if tokens[*i] != Token::And {
            break;
        }
        *i += 1;
        parts.push(parse_factor(tokens, i)?);
    }
    Some(if parts.len() == 1 {
        parts.pop().unwrap()
    } else {
        Expr::And(parts)
    })
}

fn parse_factor(tokens: &[Token], i: &mut usize) -> Option<Expr> {
    if *i >= tokens.len() {
        return None;
    }
    match &tokens[*i] {
        Token::Code(c) => {
            *i += 1;
            Some(Expr::Code(c.clone()))
        }
        Token::LParen => {
            *i += 1;
            let inner = parse_or(tokens, i)?;
            if *i < tokens.len() && tokens[*i] == Token::RParen {
                *i += 1;
            }
            Some(inner)
        }
        _ => None,
    }
}

fn flatten(expr: Expr) -> Expr {
    match expr {
        Expr::And(parts) => {
            let mut flat = Vec::new();
            for p in parts {
                match flatten(p) {
                    Expr::And(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            if flat.len() == 1 {
                flat.pop().unwrap()
            } else {
                Expr::And(flat)
            }
        }
        Expr::Or(parts) => {
            let mut flat = Vec::new();
            for p in parts {
                match flatten(p) {
                    Expr::Or(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            if flat.len() == 1 {
                flat.pop().unwrap()
            } else {
                Expr::Or(flat)
            }
        }
        other => other,
    }
}

fn expr_satisfied(expr: &Expr, plan: &HashSet<String>) -> bool {
    match expr {
        Expr::Code(c) => course_relations::set_contains_equiv(plan, c),
        Expr::And(parts) => parts.iter().all(|p| expr_satisfied(p, plan)),
        Expr::Or(parts) => parts.iter().any(|p| expr_satisfied(p, plan)),
    }
}

/// Human-readable missing fragment, e.g. `CIS 1210` or `ACCT 6110 or ACCT 6130`.
fn missing_fragment(expr: &Expr, plan: &HashSet<String>) -> Option<String> {
    if expr_satisfied(expr, plan) {
        return None;
    }
    match expr {
        Expr::Code(c) => Some(c.clone()),
        Expr::And(parts) => {
            let miss: Vec<String> = parts
                .iter()
                .filter_map(|p| missing_fragment(p, plan))
                .collect();
            if miss.is_empty() {
                None
            } else {
                Some(join_and(&miss))
            }
        }
        Expr::Or(parts) => {
            // None satisfied — list alternatives.
            let alts: Vec<String> = parts
                .iter()
                .map(|p| match p {
                    Expr::Code(c) => c.clone(),
                    other => missing_fragment(other, &HashSet::new()).unwrap_or_else(|| {
                        // Fallback: show nested structure roughly
                        format_expr(other)
                    }),
                })
                .collect();
            if alts.is_empty() {
                None
            } else {
                Some(join_or(&alts))
            }
        }
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Code(c) => c.clone(),
        Expr::And(parts) => join_and(&parts.iter().map(format_expr).collect::<Vec<_>>()),
        Expr::Or(parts) => join_or(&parts.iter().map(format_expr).collect::<Vec<_>>()),
    }
}

fn join_and(parts: &[String]) -> String {
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        2 => format!("{} and {}", parts[0], parts[1]),
        _ => {
            let last = parts.last().unwrap();
            format!("{}, and {}", parts[..parts.len() - 1].join(", "), last)
        }
    }
}

fn join_or(parts: &[String]) -> String {
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        2 => format!("{} or {}", parts[0], parts[1]),
        _ => {
            let last = parts.last().unwrap();
            format!("{}, or {}", parts[..parts.len() - 1].join(", "), last)
        }
    }
}

/// Courses in `subject_codes` that have unsatisfied catalog prerequisites
/// relative to `plan_codes` (typically subjects = taken/frozen; plan = full set).
/// Returns `(course_id, message)` pairs (one per subject with missing prereqs).
pub fn missing_prereq_messages(
    subject_codes: &[String],
    plan_codes: &[String],
) -> Vec<(String, String)> {
    let map = prereq_map();
    let plan: HashSet<String> = plan_codes
        .iter()
        .filter(|c| course::is_valid_course_code(c))
        .map(|c| course_relations::normalize_code(c))
        .collect();

    let mut seen_courses: HashSet<String> = HashSet::new();
    let mut out = Vec::new();

    for raw in subject_codes {
        if !course::is_valid_course_code(raw) {
            continue;
        }
        let code = course_relations::normalize_code(raw);
        let canon = course_relations::canonical(&code);
        if !seen_courses.insert(canon.clone()) {
            continue;
        }
        let subject = code.clone();

        let Some(expr) = map
            .get(&code)
            .or_else(|| map.get(&canon))
            .or_else(|| {
                course_relations::aliases(&code)
                    .iter()
                    .find_map(|a| map.get(a))
            })
        else {
            continue;
        };

        if expr_satisfied(expr, &plan) {
            continue;
        }
        let Some(fragment) = missing_fragment(expr, &plan) else {
            continue;
        };
        let message = if fragment.contains(" and ") || fragment.contains(" or ") {
            format!("{subject} is missing prerequisites {fragment}.")
        } else {
            format!("{subject} is missing prerequisite {fragment}.")
        };
        out.push((subject, message));
    }

    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_glued_and() {
        let e = parse_prereq("CIS\u{00a0}1200ANDCIS\u{00a0}1600").unwrap();
        assert_eq!(
            e,
            Expr::And(vec![
                Expr::Code("CIS 1200".into()),
                Expr::Code("CIS 1600".into()),
            ])
        );
    }

    #[test]
    fn parses_or_group() {
        let e = parse_prereq("ACCT 6110ORACCT 6130").unwrap();
        assert_eq!(
            e,
            Expr::Or(vec![
                Expr::Code("ACCT 6110".into()),
                Expr::Code("ACCT 6130".into()),
            ])
        );
    }

    #[test]
    fn cis_3200_missing_both() {
        let msgs = missing_prereq_messages(&["CIS 3200".into()], &["CIS 3200".into()]);
        assert!(
            msgs.iter().any(|(c, m)| c == "CIS 3200"
                && m.contains("CIS 1210")
                && m.contains("CIS 2620")),
            "{msgs:?}"
        );
    }

    #[test]
    fn cis_3200_satisfied_when_prereqs_taken() {
        let subjects = vec!["CIS 3200".into()];
        let plan = vec![
            "CIS 3200".into(),
            "CIS 1210".into(),
            "CIS 2620".into(),
        ];
        let msgs = missing_prereq_messages(&subjects, &plan);
        assert!(
            !msgs.iter().any(|(c, _)| c == "CIS 3200"),
            "{msgs:?}"
        );
    }
}
