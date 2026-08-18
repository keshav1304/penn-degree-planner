//! Cross-list (`also_offered_as`) and mutually exclusive course relations.
//!
//! Built once from the embedded catalog. Codes are normalized (NBSP → space) before indexing.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::course;
use crate::penn_data::courses_data;

static RELATIONS: OnceLock<CourseRelations> = OnceLock::new();

#[derive(Debug, Default)]
pub struct CourseRelations {
    /// Normalized code → lex-min canonical of its alias cluster.
    canonical_of: HashMap<String, String>,
    /// Canonical → all normalized codes in the cluster (sorted, includes canonical).
    clusters: HashMap<String, Vec<String>>,
    /// Normalized code → mutex partners (normalized), closed under aliases, sorted, deduped.
    mutex: HashMap<String, Vec<String>>,
}

impl CourseRelations {
    pub fn canonical(&self, code: &str) -> String {
        if let Some(c) = self.canonical_of.get(code) {
            return c.clone();
        }
        let n = normalize_code(code);
        self.canonical_of.get(&n).cloned().unwrap_or(n)
    }

    pub fn aliases(&self, code: &str) -> &[String] {
        if let Some(canon) = self.canonical_of.get(code) {
            return self
                .clusters
                .get(canon.as_str())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
        }
        let n = normalize_code(code);
        let Some(canon) = self.canonical_of.get(&n) else {
            return &[];
        };
        self.clusters
            .get(canon.as_str())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn equivalent(&self, a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }
        match (self.canonical_of.get(a), self.canonical_of.get(b)) {
            (Some(ca), Some(cb)) => ca == cb,
            _ => self.canonical(a) == self.canonical(b),
        }
    }

    pub fn mutex_partners(&self, code: &str) -> &[String] {
        if let Some(p) = self.mutex.get(code) {
            return p.as_slice();
        }
        let n = normalize_code(code);
        self.mutex
            .get(&n)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn codes_conflict(&self, a: &str, b: &str) -> bool {
        if self.equivalent(a, b) {
            return false;
        }
        self.mutex_partners(a)
            .iter()
            .any(|p| p == b || self.equivalent(p, b))
    }
}

/// NBSP → ordinary space, trim.
pub fn normalize_code(s: &str) -> String {
    s.chars()
        .map(|c| if c == '\u{00a0}' { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn relations() -> &'static CourseRelations {
    RELATIONS.get_or_init(build_relations)
}

pub fn canonical(code: &str) -> String {
    relations().canonical(code)
}

pub fn aliases(code: &str) -> &'static [String] {
    relations().aliases(code)
}

pub fn equivalent(a: &str, b: &str) -> bool {
    relations().equivalent(a, b)
}

pub fn mutex_partners(code: &str) -> &'static [String] {
    relations().mutex_partners(code)
}

pub fn codes_conflict(a: &str, b: &str) -> bool {
    relations().codes_conflict(a, b)
}

pub fn set_contains_equiv(set: &HashSet<String>, code: &str) -> bool {
    if set.contains(code) {
        return true;
    }
    let n = normalize_code(code);
    if set.contains(&n) {
        return true;
    }
    for a in aliases(code) {
        if set.contains(a) {
            return true;
        }
    }
    // Also check if any set member is equivalent (set may hold unnormalized or other aliases).
    set.iter().any(|s| equivalent(s, code))
}

pub fn vec_contains_equiv(list: &[String], code: &str) -> bool {
    list.iter().any(|s| equivalent(s, code))
}

/// Remove every code in `pool` that is equivalent to any code in `used`.
pub fn retain_without_equiv(pool: &mut Vec<String>, used: &[String]) {
    pool.retain(|x| !used.iter().any(|u| equivalent(x, u)));
}

/// First taken entry that matches any candidate (pool/taken order preserved).
pub fn taken_hit<'a>(taken: &'a [String], candidates: &[String]) -> Option<&'a String> {
    for t in taken {
        if candidates.iter().any(|c| equivalent(t, c)) {
            return Some(t);
        }
    }
    None
}

/// Prefer a possibility spelling when it is equivalent to the taken code.
pub fn display_spelling_for_match(taken_code: &str, possibilities: &[String]) -> String {
    for p in possibilities {
        if equivalent(taken_code, p) {
            return p.clone();
        }
    }
    taken_code.to_string()
}

fn parse_code_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(normalize_code)
        .filter(|s| !s.is_empty() && course::is_valid_course_code(s))
        .collect()
}

fn build_relations() -> CourseRelations {
    let courses = courses_data::courses();
    let known: HashSet<String> = courses
        .iter()
        .map(|c| normalize_code(&c.course_code))
        .filter(|c| course::is_valid_course_code(c))
        .collect();

    struct UnionFind {
        parent: HashMap<String, String>,
    }

    impl UnionFind {
        fn new() -> Self {
            Self {
                parent: HashMap::new(),
            }
        }

        fn ensure(&mut self, x: &str) {
            self.parent
                .entry(x.to_string())
                .or_insert_with(|| x.to_string());
        }

        fn find(&mut self, x: &str) -> String {
            self.ensure(x);
            let mut root = x.to_string();
            loop {
                let p = self.parent.get(&root).cloned().unwrap_or_else(|| root.clone());
                if p == root {
                    break;
                }
                root = p;
            }
            let mut cur = x.to_string();
            while let Some(p) = self.parent.get(&cur).cloned() {
                if p == cur {
                    break;
                }
                self.parent.insert(cur.clone(), root.clone());
                cur = p;
            }
            root
        }

        fn union(&mut self, a: &str, b: &str) {
            let ra = self.find(a);
            let rb = self.find(b);
            if ra == rb {
                return;
            }
            if ra < rb {
                self.parent.insert(rb, ra);
            } else {
                self.parent.insert(ra, rb);
            }
        }
    }

    let mut uf = UnionFind::new();
    for c in courses {
        let self_code = normalize_code(&c.course_code);
        if !course::is_valid_course_code(&self_code) {
            continue;
        }
        uf.ensure(&self_code);
        if let Some(raw) = &c.also_offered_as {
            for other in parse_code_list(raw) {
                if !known.contains(&other) {
                    continue;
                }
                uf.union(&self_code, &other);
            }
        }
    }

    let mut members: HashMap<String, Vec<String>> = HashMap::new();
    let codes: Vec<String> = uf.parent.keys().cloned().collect();
    for code in &codes {
        let root = uf.find(code);
        members.entry(root).or_default().push(code.clone());
    }

    let mut canonical_of = HashMap::new();
    let mut clusters = HashMap::new();
    for (_root, mut group) in members {
        group.sort();
        group.dedup();
        let canon = group[0].clone();
        for g in &group {
            canonical_of.insert(g.clone(), canon.clone());
        }
        clusters.insert(canon, group);
    }

    let mut raw_mutex: HashMap<String, HashSet<String>> = HashMap::new();
    for c in courses {
        let self_code = normalize_code(&c.course_code);
        if !course::is_valid_course_code(&self_code) {
            continue;
        }
        let Some(raw) = &c.mutually_exclusive else {
            continue;
        };
        for other in parse_code_list(raw) {
            if !known.contains(&other) || other == self_code {
                continue;
            }
            raw_mutex
                .entry(self_code.clone())
                .or_default()
                .insert(other.clone());
            raw_mutex
                .entry(other.clone())
                .or_default()
                .insert(self_code.clone());
        }
    }

    let mut mutex: HashMap<String, Vec<String>> = HashMap::new();
    let all_codes: Vec<String> = canonical_of.keys().cloned().collect();
    for code in &all_codes {
        let mut partners: HashSet<String> = HashSet::new();
        let canon = canonical_of.get(code).cloned().unwrap_or_else(|| code.clone());
        let cluster = clusters
            .get(&canon)
            .cloned()
            .unwrap_or_else(|| vec![code.clone()]);
        for member in &cluster {
            if let Some(raws) = raw_mutex.get(member) {
                for r in raws {
                    let partner_canon = canonical_of.get(r).cloned().unwrap_or_else(|| r.clone());
                    let partner_cluster = clusters
                        .get(&partner_canon)
                        .cloned()
                        .unwrap_or_else(|| vec![r.clone()]);
                    for p in partner_cluster {
                        if !cluster.contains(&p) {
                            partners.insert(p);
                        }
                    }
                }
            }
        }
        if partners.is_empty() {
            continue;
        }
        let mut list: Vec<String> = partners.into_iter().collect();
        list.sort();
        mutex.insert(code.clone(), list);
    }

    CourseRelations {
        canonical_of,
        clusters,
        mutex,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_nbsp_and_spaces() {
        assert_eq!(normalize_code("BEPP\u{00a0}2110"), "BEPP 2110");
        assert_eq!(normalize_code("  CIS   1200  "), "CIS 1200");
    }

    #[test]
    fn parse_comma_list() {
        let list = parse_code_list("SOCI\u{00a0}2940,URBS\u{00a0}0010");
        assert!(list.contains(&"SOCI 2940".to_string()));
        assert!(list.contains(&"URBS 0010".to_string()));
    }

    #[test]
    fn acct_bepp_also_offered_equivalent() {
        assert!(equivalent("ACCT 2110", "BEPP 2110"));
        assert_eq!(canonical("ACCT 2110"), canonical("BEPP 2110"));
    }

    #[test]
    fn cis_mutex_pair() {
        assert!(codes_conflict("CIS 5190", "CIS 4190"));
        assert!(!codes_conflict("CIS 5190", "CIS 5190"));
    }

    #[test]
    fn mutex_closed_under_aliases() {
        // ACCT 2640 also BEPP 2640; ACCT 2640 mutex BEPP 7640
        assert!(codes_conflict("BEPP 2640", "BEPP 7640") || codes_conflict("ACCT 2640", "BEPP 7640"));
        assert!(codes_conflict("ACCT 2640", "BEPP 7640"));
    }
}
