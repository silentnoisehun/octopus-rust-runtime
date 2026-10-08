//! GP-echo interference engine
//!
//! Calculates wave interference score between task query and template:
//! I = A_q * A_t * cos(theta) * B_tech * B_pattern
//! Resonance threshold: I > 0.72

use std::collections::HashSet;

pub const RESONANCE_THRESHOLD: f64 = 0.72;

pub type WaveCandidate = (usize, Vec<String>, f64, Vec<String>, Vec<String>);

#[derive(Debug, Clone)]
pub struct InterferenceMatch {
    pub template_id: usize,
    pub score: f64,
    pub is_resonant: bool,
}

pub struct WaveEngine;

impl WaveEngine {
    /// Calculate GP-echo interference score
    pub fn calculate_interference(
        query: &str,
        template_keywords: &[String],
        template_amplitude: f64,
        template_tech: &[String],
        template_patterns: &[String],
    ) -> f64 {
        let a_q = 1.0; // Default query amplitude
        let a_t = template_amplitude.clamp(0.1, 1.0);

        let query_tokens: HashSet<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if query_tokens.is_empty() || template_keywords.is_empty() {
            return 0.0;
        }

        let template_set: HashSet<String> =
            template_keywords.iter().map(|s| s.to_lowercase()).collect();

        let intersection = query_tokens.intersection(&template_set).count();

        let cos_theta = if !query_tokens.is_empty() && !template_set.is_empty() {
            intersection as f64 / (query_tokens.len() as f64 * template_set.len() as f64).sqrt()
        } else {
            0.0
        };

        // Tech stack bonus
        let mut b_tech: f64 = 1.0;
        for tech in template_tech {
            if query_tokens.contains(&tech.to_lowercase()) {
                b_tech += 0.15;
            }
        }
        b_tech = b_tech.min(1.5);

        // Pattern bonus
        let mut b_pattern: f64 = 1.0;
        for pattern in template_patterns {
            if query_tokens.contains(&pattern.to_lowercase()) {
                b_pattern += 0.10;
            }
        }
        b_pattern = b_pattern.min(1.3);

        a_q * a_t * cos_theta * b_tech * b_pattern
    }

    /// Search candidates and return best match
    pub fn find_best_match(query: &str, candidates: &[WaveCandidate]) -> Option<InterferenceMatch> {
        let mut best: Option<InterferenceMatch> = None;

        for &(id, ref keywords, amp, ref tech, ref patterns) in candidates {
            let score = Self::calculate_interference(query, keywords, amp, tech, patterns);
            if score >= RESONANCE_THRESHOLD && best.as_ref().is_none_or(|b| score > b.score) {
                best = Some(InterferenceMatch {
                    template_id: id,
                    score,
                    is_resonant: true,
                });
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interference_threshold() {
        let keywords = vec!["rust".to_string(), "http".to_string(), "axum".to_string()];
        let tech = vec!["axum".to_string(), "tokio".to_string()];
        let patterns = vec!["server".to_string()];

        let score = WaveEngine::calculate_interference(
            "rust http server axum tokio",
            &keywords,
            0.9,
            &tech,
            &patterns,
        );

        assert!(score >= RESONANCE_THRESHOLD, "score: {score}");
    }
}
