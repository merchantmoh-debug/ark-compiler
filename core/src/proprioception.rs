/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 *
 * Proprioception — Digital Self-Sensing
 *
 * Ported from Remember-Me-AI's Proprioception class.
 * Audits output confidence, hallucination risk, and system fatigue.
 *
 * "IF (Confidence < 90%): DELETE OUTPUT AND REGENERATE."
 */

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// ProprioceptionAudit — the output of audit_output()
// ---------------------------------------------------------------------------

/// Result of a proprioceptive audit on generated output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProprioceptionAudit {
    /// Output confidence (0.0 – 1.0).
    pub confidence: f64,
    /// Hallucination risk (1.0 - confidence).
    pub hallucination_risk: f64,
    /// Whether the output contains executable code.
    pub executable: bool,
    /// Whether the output contains citations.
    pub cited: bool,
    /// System fatigue level (0.0 = fresh, 1.0 = exhausted).
    pub fatigue: f64,
    /// Energy level (100 - fatigue*100).
    pub battery_level: f64,
    /// Whether the output should be regenerated (confidence < 0.9).
    pub regenerate: bool,
}

impl ProprioceptionAudit {
    /// Returns the telemetry footer string.
    pub fn telemetry_signature(&self) -> String {
        format!(
            "\n\n[DIGITAL PROPRIOCEPTION] CONFIDENCE: {:.1}% | BATTERY: {:.0}% | HALLUCINATION RISK: {:.1}%",
            self.confidence * 100.0,
            self.battery_level,
            self.hallucination_risk * 100.0,
        )
    }
}

// ---------------------------------------------------------------------------
// Hedging / uncertainty word sets
// ---------------------------------------------------------------------------

static UNCERTAINTY_PHRASES: OnceLock<Vec<&'static str>> = OnceLock::new();

fn uncertainty_phrases() -> &'static Vec<&'static str> {
    UNCERTAINTY_PHRASES.get_or_init(|| {
        vec![
            "i'm not sure",
            "i don't know",
            "as an ai",
            "i cannot",
            "i am unable",
        ]
    })
}

static HEDGING_PHRASES: OnceLock<Vec<&'static str>> = OnceLock::new();

fn hedging_phrases() -> &'static Vec<&'static str> {
    HEDGING_PHRASES.get_or_init(|| vec!["however", "it depends", "on the other hand", "arguably"])
}

// ---------------------------------------------------------------------------
// Proprioception
// ---------------------------------------------------------------------------

/// Digital Proprioception — self-sensing for generated outputs.
///
/// Audits confidence, hallucination risk, and system fatigue.
/// Signals when an output should be regenerated (Persona Law 4).
pub struct Proprioception {
    /// Injected fatigue level (0.0 – 1.0). Updated by the runtime.
    fatigue: f64,
}

impl Default for Proprioception {
    fn default() -> Self {
        Self::new()
    }
}

impl Proprioception {
    /// Create a new Proprioception sensor with zero fatigue.
    pub fn new() -> Self {
        Self { fatigue: 0.0 }
    }

    /// Create with a specific fatigue level (for testing).
    pub fn with_fatigue(fatigue: f64) -> Self {
        Self {
            fatigue: fatigue.clamp(0.0, 1.0),
        }
    }

    /// Update the fatigue level (called by the runtime when resources change).
    pub fn set_fatigue(&mut self, fatigue: f64) {
        self.fatigue = fatigue.clamp(0.0, 1.0);
    }

    /// Get current fatigue level.
    pub fn fatigue(&self) -> f64 {
        self.fatigue
    }

    /// Audit a generated response against its context.
    ///
    /// Returns a `ProprioceptionAudit` with confidence and regeneration signal.
    pub fn audit_output(&self, response: &str, _context: &str) -> ProprioceptionAudit {
        let response_lower = response.to_lowercase();

        // Citation check
        let has_citation =
            (response.contains('[') && response.contains(']')) || response.contains("Source:");

        // Code check
        let has_code = response.contains("```");

        // Build confidence score
        let mut confidence: f64 = 0.7; // Base

        // Positive signals
        if response.len() > 200 {
            confidence += 0.1;
        }
        if has_citation {
            confidence += 0.1;
        }
        if has_code {
            confidence += 0.15;
        }

        // Negative signals: uncertainty
        for phrase in uncertainty_phrases() {
            if response_lower.contains(phrase) {
                confidence -= 0.2;
            }
        }

        // Negative signals: hedging (Anti-Hedging Law)
        for phrase in hedging_phrases() {
            if response_lower.contains(phrase) {
                confidence -= 0.1;
            }
        }

        // Negative signal: mock content
        if response_lower.contains("mock") {
            confidence -= 0.1;
        }

        // Clamp
        confidence = confidence.clamp(0.0, 1.0);

        // Regeneration threshold: Persona Law 4
        let regenerate = confidence < 0.9;

        ProprioceptionAudit {
            confidence,
            hallucination_risk: 1.0 - confidence,
            executable: has_code,
            cited: has_citation,
            fatigue: self.fatigue,
            battery_level: 100.0 - (self.fatigue * 100.0),
            regenerate,
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_confidence_response() {
        let p = Proprioception::new();
        let response = "The answer is X [Source: RFC 1234]. Here is the implementation:\n```rust\nfn foo() {}\n```\nThis approach works because of Y and Z, which are documented extensively in the specification.";
        let audit = p.audit_output(response, "context");
        assert!(
            audit.confidence > 0.9,
            "Should be high confidence with citation+code+length, got {}",
            audit.confidence
        );
        assert!(!audit.regenerate);
        assert!(audit.cited);
        assert!(audit.executable);
    }

    #[test]
    fn test_low_confidence_uncertain() {
        let p = Proprioception::new();
        let audit = p.audit_output("I'm not sure, as an AI I cannot verify this", "context");
        assert!(
            audit.confidence < 0.5,
            "Should be low confidence, got {}",
            audit.confidence
        );
        assert!(audit.regenerate);
    }

    #[test]
    fn test_hedging_penalty() {
        let p = Proprioception::new();
        let a1 = p.audit_output("The answer is X.", "context");
        let a2 = p.audit_output("The answer is X, however it depends on context.", "context");
        assert!(
            a2.confidence < a1.confidence,
            "Hedging should reduce confidence: {} vs {}",
            a2.confidence,
            a1.confidence
        );
    }

    #[test]
    fn test_fatigue_affects_battery() {
        let p = Proprioception::with_fatigue(0.8);
        let audit = p.audit_output("Response", "context");
        assert!(
            (audit.battery_level - 20.0).abs() < 0.1,
            "Battery should be 20% at 80% fatigue, got {}",
            audit.battery_level
        );
    }

    #[test]
    fn test_telemetry_signature() {
        let p = Proprioception::new();
        let audit = p.audit_output("Some response", "context");
        let sig = audit.telemetry_signature();
        assert!(sig.contains("DIGITAL PROPRIOCEPTION"));
        assert!(sig.contains("CONFIDENCE"));
        assert!(sig.contains("HALLUCINATION RISK"));
    }

    #[test]
    fn test_confidence_clamped() {
        let p = Proprioception::new();
        // Lots of negative signals
        let audit = p.audit_output(
            "I'm not sure, I don't know, as an AI I cannot, however, it depends, mock data",
            "context",
        );
        assert!(audit.confidence >= 0.0, "Confidence should be >= 0.0");
        assert!(audit.confidence <= 1.0, "Confidence should be <= 1.0");
    }
}
