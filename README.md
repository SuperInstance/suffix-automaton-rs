# suffix-automaton-rs

Suffix automaton (SAM) — the smallest DFA accepting all substrings of a string. O(n) construction. Pure `std`, no dependencies.

## Features

- `SuffixAutomaton::new(s)` — O(n) build
- `.distinct_substrings()` — count distinct non-empty substrings
- `.contains(pattern)` — O(m) substring test
- `.occurrences(pattern)` — count occurrences via suffix-link propagation
- `.all_substrings()` — enumerate every distinct substring
