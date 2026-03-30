use std::cmp::min;

/// Find the most similar alias using Levenshtein distance.
pub fn find_similar_alias(alias: &str, aliases: &[String]) -> String {
    let mut min_distance = usize::MAX;
    let mut similar_alias = String::new();

    for candidate in aliases {
        let distance = levenshtein_distance(alias, candidate);
        if distance < min_distance {
            min_distance = distance;
            similar_alias = candidate.clone();
        }
    }

    similar_alias
}

/// Calculate the Levenshtein distance between two strings.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let mut dp = vec![vec![0; s2_chars.len() + 1]; s1_chars.len() + 1];

    for i in 0..=s1_chars.len() {
        for j in 0..=s2_chars.len() {
            if i == 0 {
                dp[i][j] = j;
            } else if j == 0 {
                dp[i][j] = i;
            } else {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                dp[i][j] = min(
                    dp[i - 1][j - 1] + cost,
                    min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                );
            }
        }
    }
    dp[s1_chars.len()][s2_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "world"), 4);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
    }

    #[test]
    fn test_find_similar_alias() {
        let aliases = vec![
            "my-local-mysql".to_string(),
            "my-local-redis".to_string(),
            "my-local-mongo".to_string(),
        ];
        assert_eq!(find_similar_alias("my-local-mysq", &aliases), "my-local-mysql");
        assert_eq!(find_similar_alias("my-local-redi", &aliases), "my-local-redis");
    }
}
