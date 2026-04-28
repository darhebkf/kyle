use strsim::damerau_levenshtein;

const MAX_DISTANCE: usize = 2;
const MIN_MATCH_RATIO: f64 = 0.5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub candidate: String,
    pub distance: usize,
}

/// Closest candidates by edit distance, sorted ascending.
/// Inputs containing `:` are matched per-segment so segment counts
/// must align (avoids `b:t` proposing `backend:test`).
pub fn suggest(input: &str, candidates: &[String]) -> Vec<Suggestion> {
    if input.is_empty() || candidates.is_empty() {
        return Vec::new();
    }

    if input.contains(':') {
        return suggest_namespaced(input, candidates);
    }

    let mut scored: Vec<Suggestion> = candidates.iter().filter_map(|c| score(input, c)).collect();
    scored.sort_by_key(|s| s.distance);
    scored
}

/// Best match only when there is a strict winner; `None` on tie or
/// no match — caller falls back to suggest mode rather than guessing.
pub fn best_unambiguous(input: &str, candidates: &[String]) -> Option<Suggestion> {
    let scored = suggest(input, candidates);
    match scored.as_slice() {
        [only] => Some(only.clone()),
        [first, second, ..] if first.distance < second.distance => Some(first.clone()),
        _ => None,
    }
}

fn score(input: &str, candidate: &str) -> Option<Suggestion> {
    let distance = damerau_levenshtein(input, candidate);
    if distance == 0 || distance > MAX_DISTANCE {
        return None;
    }
    let input_len = input.chars().count() as f64;
    let matched = (input.chars().count() - distance) as f64;
    if matched / input_len < MIN_MATCH_RATIO {
        return None;
    }
    Some(Suggestion {
        candidate: candidate.to_string(),
        distance,
    })
}

fn suggest_namespaced(input: &str, candidates: &[String]) -> Vec<Suggestion> {
    let input_segs: Vec<&str> = input.split(':').collect();

    let mut scored: Vec<Suggestion> = candidates
        .iter()
        .filter_map(|candidate| {
            let cand_segs: Vec<&str> = candidate.split(':').collect();
            if cand_segs.len() != input_segs.len() {
                return None;
            }
            let mut total = 0usize;
            for (i, seg) in input_segs.iter().enumerate() {
                total += damerau_levenshtein(seg, cand_segs[i]);
                if total > MAX_DISTANCE {
                    return None;
                }
            }
            if total == 0 {
                return None;
            }
            Some(Suggestion {
                candidate: candidate.clone(),
                distance: total,
            })
        })
        .collect();
    scored.sort_by_key(|s| s.distance);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cands(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn close_typo_returns_match() {
        let result = suggest("buld", &cands(&["build", "test", "deploy"]));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].candidate, "build");
        assert_eq!(result[0].distance, 1);
    }

    #[test]
    fn exact_match_is_excluded() {
        let result = suggest("build", &cands(&["build", "test"]));
        assert!(result.is_empty());
    }

    #[test]
    fn distant_input_returns_nothing() {
        let result = suggest("xyz", &cands(&["build", "test", "deploy"]));
        assert!(result.is_empty());
    }

    #[test]
    fn tiny_input_does_not_match_long_candidate() {
        let result = suggest("x", &cands(&["build", "test"]));
        assert!(result.is_empty());
    }

    #[test]
    fn results_sorted_by_distance() {
        let result = suggest("buil", &cands(&["build", "bind", "boil"]));
        assert!(!result.is_empty());
        for w in result.windows(2) {
            assert!(w[0].distance <= w[1].distance);
        }
    }

    #[test]
    fn tie_at_best_distance_is_ambiguous() {
        let result = best_unambiguous("bild", &cands(&["build", "bind"]));
        assert!(result.is_none());
    }

    #[test]
    fn unambiguous_winner_is_returned() {
        let result = best_unambiguous("buld", &cands(&["build", "deploy", "test"]));
        assert_eq!(result.unwrap().candidate, "build");
    }

    #[test]
    fn namespaced_matches_per_segment() {
        let result = suggest(
            "backed:tst",
            &cands(&["backend:test", "backend:build", "frontend:test"]),
        );
        assert!(!result.is_empty());
        assert_eq!(result[0].candidate, "backend:test");
    }

    #[test]
    fn namespaced_segment_count_must_match() {
        let result = suggest("backed:tst", &cands(&["backend:ccm:test"]));
        assert!(result.is_empty());
    }

    #[test]
    fn namespaced_total_distance_capped() {
        let result = suggest("xxxx:yyyy", &cands(&["build:test"]));
        assert!(result.is_empty());
    }

    #[test]
    fn empty_inputs_return_empty() {
        assert!(suggest("", &cands(&["build"])).is_empty());
        assert!(suggest("build", &[]).is_empty());
    }

    // Damerau-Levenshtein collapses single transpositions to distance 1.
    #[test]
    fn transposition_is_a_single_edit() {
        let result = suggest("bulid", &cands(&["build"]));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].distance, 1);
    }
}
