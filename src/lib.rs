//! Suffix automaton (SAM) — the smallest DFA accepting all substrings of a string.
//!
//! Provides O(n) construction, distinct substring enumeration, and occurrence
//! counting for each equivalence class.
//!
//! # Examples
//!
//! ```
//! use suffix_automaton::SuffixAutomaton;
//!
//! let sam = SuffixAutomaton::new(b"abcbc");
//! assert_eq!(sam.distinct_substrings(), 12);
//! assert!(sam.contains(b"cbc"));
//! assert!(!sam.contains(b"xyz"));
//! ```

use std::collections::HashMap;

// ── state ─────────────────────────────────────────────────────────────────────

struct State {
    len: usize,
    link: usize,
    next: HashMap<u8, usize>,
    cnt: usize,
}

impl State {
    fn new(len: usize, link: usize) -> Self {
        Self { len, link, next: HashMap::new(), cnt: 0 }
    }
}

// ── SuffixAutomaton ───────────────────────────────────────────────────────────

/// Suffix automaton built over a byte string.
pub struct SuffixAutomaton {
    states: Vec<State>,
    last: usize,
    size: usize,
}

impl SuffixAutomaton {
    /// Build a suffix automaton for the given byte string in O(n).
    pub fn new(s: &[u8]) -> Self {
        let mut sam = Self {
            states: Vec::with_capacity(2 * s.len() + 2),
            last: 0,
            size: s.len(),
        };
        // State 0: initial state
        sam.states.push(State::new(0, usize::MAX));
        for &c in s {
            sam.extend(c);
        }
        sam
    }

    fn extend(&mut self, c: u8) {
        let cur = self.states.len();
        self.states.push(State::new(self.states[self.last].len + 1, usize::MAX));
        self.states[cur].cnt = 1; // each new state corresponds to an end position

        let mut p = self.last;
        while p != usize::MAX && !self.states[p].next.contains_key(&c) {
            self.states[p].next.insert(c, cur);
            p = self.states[p].link;
        }

        if p == usize::MAX {
            self.states[cur].link = 0; // initial state
        } else {
            let q = self.states[p].next[&c];
            if self.states[p].len + 1 == self.states[q].len {
                self.states[cur].link = q;
            } else {
                let clone = self.states.len();
                let q_len = self.states[q].len;
                let q_link = self.states[q].link;
                let q_next = self.states[q].next.clone();
                self.states.push(State { len: self.states[p].len + 1, link: q_link, next: q_next, cnt: 0 });
                // clone's len is now states[p].len + 1
                while p != usize::MAX {
                    let entry = self.states[p].next.get(&c).copied();
                    if entry == Some(q) {
                        self.states[p].next.insert(c, clone);
                        p = self.states[p].link;
                    } else {
                        break;
                    }
                }
                self.states[q].link = clone;
                self.states[cur].link = clone;
                let _ = q_len; // used above to keep the clone's len field
            }
        }
        self.last = cur;
    }

    /// Propagate counts up suffix links (topological order by `len`).
    fn propagate_counts(&self) -> Vec<usize> {
        let n = self.states.len();
        let mut cnt: Vec<usize> = self.states.iter().map(|s| s.cnt).collect();
        // Sort states by len descending (topological order of suffix-link tree)
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| self.states[b].len.cmp(&self.states[a].len));
        for &v in &order {
            let link = self.states[v].link;
            if link != usize::MAX {
                cnt[link] += cnt[v];
            }
        }
        cnt
    }

    /// Number of distinct non-empty substrings.
    pub fn distinct_substrings(&self) -> u64 {
        self.states
            .iter()
            .skip(1) // skip initial state
            .map(|s| {
                let parent_len = if s.link == usize::MAX { 0 } else { self.states[s.link].len };
                (s.len - parent_len) as u64
            })
            .sum()
    }

    /// Check whether `pattern` is a substring of the original string.
    pub fn contains(&self, pattern: &[u8]) -> bool {
        let mut cur = 0usize;
        for &c in pattern {
            match self.states[cur].next.get(&c) {
                Some(&next) => cur = next,
                None => return false,
            }
        }
        true
    }

    /// Count occurrences of `pattern` in the original string.
    pub fn occurrences(&self, pattern: &[u8]) -> usize {
        if pattern.is_empty() {
            return self.size + 1; // every position including empty
        }
        let mut cur = 0usize;
        for &c in pattern {
            match self.states[cur].next.get(&c) {
                Some(&next) => cur = next,
                None => return 0,
            }
        }
        self.propagate_counts()[cur]
    }

    /// Number of states in the automaton (excluding the initial state).
    pub fn num_states(&self) -> usize {
        self.states.len() - 1
    }

    /// Number of transitions in the automaton.
    pub fn num_transitions(&self) -> usize {
        self.states.iter().map(|s| s.next.len()).sum()
    }

    /// Collect all distinct substrings as byte vectors.
    pub fn all_substrings(&self) -> Vec<Vec<u8>> {
        let mut result = Vec::new();
        let mut stack: Vec<(usize, Vec<u8>)> = vec![(0, vec![])];
        while let Some((state, prefix)) = stack.pop() {
            if !prefix.is_empty() {
                result.push(prefix.clone());
            }
            let mut edges: Vec<(u8, usize)> =
                self.states[state].next.iter().map(|(&c, &s)| (c, s)).collect();
            edges.sort();
            for (c, next) in edges {
                let mut new_prefix = prefix.clone();
                new_prefix.push(c);
                stack.push((next, new_prefix));
            }
        }
        result
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── construction ─────────────────────────────────────────────────────────

    #[test]
    fn sam_empty() {
        let sam = SuffixAutomaton::new(b"");
        assert_eq!(sam.distinct_substrings(), 0);
        assert_eq!(sam.num_states(), 0);
    }

    #[test]
    fn sam_single_char() {
        let sam = SuffixAutomaton::new(b"a");
        assert_eq!(sam.distinct_substrings(), 1);
        assert_eq!(sam.num_states(), 1);
    }

    #[test]
    fn sam_two_distinct() {
        let sam = SuffixAutomaton::new(b"ab");
        // "a", "b", "ab" → 3 distinct substrings
        assert_eq!(sam.distinct_substrings(), 3);
    }

    #[test]
    fn sam_repeated() {
        let sam = SuffixAutomaton::new(b"aaa");
        // "a", "aa", "aaa" → 3 distinct substrings
        assert_eq!(sam.distinct_substrings(), 3);
    }

    #[test]
    fn sam_abcbc() {
        let sam = SuffixAutomaton::new(b"abcbc");
        // Total = n*(n+1)/2 - duplicates; known answer = 12
        assert_eq!(sam.distinct_substrings(), 12);
    }

    #[test]
    fn sam_banana() {
        let sam = SuffixAutomaton::new(b"banana");
        // Same as suffix array: 15
        assert_eq!(sam.distinct_substrings(), 15);
    }

    #[test]
    fn num_states_bounds() {
        let sam = SuffixAutomaton::new(b"abcde");
        // At most 2*n - 1 states for unique chars
        assert!(sam.num_states() <= 2 * 5 - 1);
    }

    // ── contains ─────────────────────────────────────────────────────────────

    #[test]
    fn contains_true() {
        let sam = SuffixAutomaton::new(b"abcbc");
        assert!(sam.contains(b"cbc"));
        assert!(sam.contains(b"abc"));
        assert!(sam.contains(b"b"));
        assert!(sam.contains(b"abcbc"));
    }

    #[test]
    fn contains_false() {
        let sam = SuffixAutomaton::new(b"abcbc");
        assert!(!sam.contains(b"xyz"));
        assert!(!sam.contains(b"abcc"));
        assert!(!sam.contains(b"abcbcd"));
    }

    #[test]
    fn contains_empty_pattern() {
        let sam = SuffixAutomaton::new(b"hello");
        assert!(sam.contains(b"")); // empty string always contained
    }

    #[test]
    fn contains_full_string() {
        let sam = SuffixAutomaton::new(b"hello");
        assert!(sam.contains(b"hello"));
    }

    // ── occurrences ──────────────────────────────────────────────────────────

    #[test]
    fn occurrences_banana_a() {
        let sam = SuffixAutomaton::new(b"banana");
        assert_eq!(sam.occurrences(b"a"), 3);
    }

    #[test]
    fn occurrences_banana_an() {
        let sam = SuffixAutomaton::new(b"banana");
        assert_eq!(sam.occurrences(b"an"), 2);
    }

    #[test]
    fn occurrences_banana_ana() {
        let sam = SuffixAutomaton::new(b"banana");
        assert_eq!(sam.occurrences(b"ana"), 2);
    }

    #[test]
    fn occurrences_not_found() {
        let sam = SuffixAutomaton::new(b"banana");
        assert_eq!(sam.occurrences(b"xyz"), 0);
    }

    #[test]
    fn occurrences_single() {
        let sam = SuffixAutomaton::new(b"banana");
        assert_eq!(sam.occurrences(b"banana"), 1);
    }

    // ── all_substrings ────────────────────────────────────────────────────────

    #[test]
    fn all_substrings_ab() {
        let sam = SuffixAutomaton::new(b"ab");
        let mut subs: Vec<String> =
            sam.all_substrings().iter().map(|v| String::from_utf8(v.clone()).unwrap()).collect();
        subs.sort();
        assert_eq!(subs, vec!["a", "ab", "b"]);
    }

    #[test]
    fn all_substrings_count_matches_distinct() {
        let sam = SuffixAutomaton::new(b"abcbc");
        assert_eq!(sam.all_substrings().len() as u64, sam.distinct_substrings());
    }

    #[test]
    fn all_substrings_no_duplicates() {
        let sam = SuffixAutomaton::new(b"aababc");
        let subs = sam.all_substrings();
        let mut deduped = subs.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(subs.len(), deduped.len());
    }

    // ── transitions ──────────────────────────────────────────────────────────

    #[test]
    fn num_transitions_nonempty() {
        let sam = SuffixAutomaton::new(b"abc");
        assert!(sam.num_transitions() >= 3);
    }
}
