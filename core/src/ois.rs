/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 *
 * OIS Module — Sovereign Cognitive Frameworks
 *
 * Ported from Remember-Me-AI's frameworks.py. Includes:
 * - OISTruthBudget: Every assumption burns capital. HALT at zero.
 * - HaiyueMicrocosm: 3-trajectory simulation (Optimistic/Neutral/Pessimistic).
 * - VelocityPhysics: Mode configs (Turtle vs Hare) governing execution params.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===========================================================================
// OIS TRUTH BUDGET
// ===========================================================================

/// Standardized cost categories for OIS deductions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OisCostType {
    /// Unverified assumption. Cost: 20.
    Assumption,
    /// Context dependency. Cost: 5.
    Context,
    /// Unproven correlation. Cost: 15.
    Correlation,
    /// Emotional reasoning driver. Cost: 25.
    EmotionalDriver,
    /// Fundamentally undecidable claim. Cost: 40.
    Undecidable,
    /// High-entropy / noisy input processing. Cost: 10.
    HighEntropy,
    /// Veto circuit triggered. Cost: 100.
    VetoTrigger,
    /// External search timed out. Cost: 15.
    SearchTimeout,
    /// Code execution failure. Cost: 10.
    CodeFailure,
    /// Dangerous code detected. Cost: 20.
    DangerousCode,
    /// Hallucination suspected. Cost: 40.
    Hallucination,
    /// Output regeneration required. Cost: 10.
    Regeneration,
}

impl OisCostType {
    /// The standardized cost for this action type.
    pub fn cost(self) -> i64 {
        match self {
            Self::Assumption => 20,
            Self::Context => 5,
            Self::Correlation => 15,
            Self::EmotionalDriver => 25,
            Self::Undecidable => 40,
            Self::HighEntropy => 10,
            Self::VetoTrigger => 100,
            Self::SearchTimeout => 15,
            Self::CodeFailure => 10,
            Self::DangerousCode => 20,
            Self::Hallucination => 40,
            Self::Regeneration => 10,
        }
    }
}

impl std::fmt::Display for OisCostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assumption => write!(f, "ASSUMPTION"),
            Self::Context => write!(f, "CONTEXT"),
            Self::Correlation => write!(f, "CORRELATION"),
            Self::EmotionalDriver => write!(f, "EMOTIONAL_DRIVER"),
            Self::Undecidable => write!(f, "UNDECIDABLE"),
            Self::HighEntropy => write!(f, "HIGH_ENTROPY"),
            Self::VetoTrigger => write!(f, "VETO_TRIGGER"),
            Self::SearchTimeout => write!(f, "SEARCH_TIMEOUT"),
            Self::CodeFailure => write!(f, "CODE_FAILURE"),
            Self::DangerousCode => write!(f, "DANGEROUS_CODE"),
            Self::Hallucination => write!(f, "HALLUCINATION"),
            Self::Regeneration => write!(f, "REGENERATION"),
        }
    }
}

/// A single deduction record in the OIS audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OisDeduction {
    /// The cost type.
    pub cost_type: OisCostType,
    /// Amount deducted.
    pub amount: i64,
    /// Human-readable reason.
    pub reason: String,
    /// Remaining budget after this deduction.
    pub remaining: i64,
}

/// OIS Truth Budget — economic logic for reasoning.
///
/// Start with 100 points. Every assumption, correlation, or speculation
/// burns capital. When budget reaches zero: **HALT**.
///
/// Framework 10 (Semantic Ledger): Every claim must PURCHASE the next
/// claim with evidence. No free moves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OisTruthBudget {
    /// Current remaining budget.
    pub budget: i64,
    /// Audit log of all deductions.
    pub history: Vec<OisDeduction>,
}

impl Default for OisTruthBudget {
    fn default() -> Self {
        Self::new(100)
    }
}

impl OisTruthBudget {
    /// Create a new OIS budget with the given initial balance.
    pub fn new(initial_budget: i64) -> Self {
        Self {
            budget: initial_budget,
            history: Vec::new(),
        }
    }

    /// Deduct a raw amount with a reason.
    pub fn deduct(&mut self, amount: i64, reason: impl Into<String>) {
        self.budget -= amount;
        self.history.push(OisDeduction {
            cost_type: OisCostType::Context, // generic
            amount,
            reason: reason.into(),
            remaining: self.budget,
        });
    }

    /// Deduct by standardized cost type.
    pub fn deduct_by_type(&mut self, cost_type: OisCostType, details: impl Into<String>) {
        let amount = cost_type.cost();
        let details = details.into();
        let reason = if details.is_empty() {
            cost_type.to_string()
        } else {
            format!("{}: {}", cost_type, details)
        };
        self.budget -= amount;
        self.history.push(OisDeduction {
            cost_type,
            amount,
            reason,
            remaining: self.budget,
        });
    }

    /// Framework 10: Semantic Ledger. A claim must PURCHASE the next claim.
    ///
    /// Returns `true` if the budget can afford the claim, `false` if depleted.
    pub fn check_ledger(&mut self, claim_cost: i64) -> bool {
        if self.budget >= claim_cost {
            self.deduct(claim_cost, "SEMANTIC_LEDGER: Claim Verification");
            true
        } else {
            false
        }
    }

    /// Returns `true` if the budget is still positive (system can continue).
    pub fn check(&self) -> bool {
        self.budget > 0
    }

    /// Returns `true` if the budget is depleted (MUST HALT).
    pub fn is_depleted(&self) -> bool {
        self.budget <= 0
    }

    /// Human-readable status.
    pub fn status(&self) -> &'static str {
        if self.budget > 0 {
            "SOUND"
        } else {
            "DEPLETED"
        }
    }
}

// ===========================================================================
// HAIYUE MICROCOSM
// ===========================================================================

/// One of the three Haiyue trajectories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trajectory {
    /// Optimistic (+1) scenario.
    Optimistic,
    /// Neutral (0) scenario.
    Neutral,
    /// Pessimistic (-1) scenario.
    Pessimistic,
}

impl std::fmt::Display for Trajectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Optimistic => write!(f, "OPTIMISTIC (+1)"),
            Self::Neutral => write!(f, "NEUTRAL (0)"),
            Self::Pessimistic => write!(f, "PESSIMISTIC (-1)"),
        }
    }
}

/// The result of a Haiyue 3-trajectory simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaiyueSimulation {
    /// The optimistic trajectory output.
    pub optimistic: String,
    /// The neutral trajectory output.
    pub neutral: String,
    /// The pessimistic trajectory output.
    pub pessimistic: String,
}

impl HaiyueSimulation {
    /// Create from a map of trajectory results.
    pub fn from_results(results: &HashMap<Trajectory, String>) -> Self {
        Self {
            optimistic: results
                .get(&Trajectory::Optimistic)
                .cloned()
                .unwrap_or_else(|| "N/A".into()),
            neutral: results
                .get(&Trajectory::Neutral)
                .cloned()
                .unwrap_or_else(|| "N/A".into()),
            pessimistic: results
                .get(&Trajectory::Pessimistic)
                .cloned()
                .unwrap_or_else(|| "N/A".into()),
        }
    }

    /// Generate the system prompt for a specific trajectory.
    pub fn trajectory_prompt(trajectory: Trajectory, user_input: &str) -> String {
        match trajectory {
            Trajectory::Optimistic => {
                format!(
                    "Simulate an OPTIMISTIC (+1) outcome/answer for: {}. Be concise.",
                    user_input
                )
            }
            Trajectory::Neutral => {
                format!(
                    "Simulate a NEUTRAL (0) outcome/answer for: {}. Be concise.",
                    user_input
                )
            }
            Trajectory::Pessimistic => {
                format!(
                    "Simulate a PESSIMISTIC (-1) outcome/answer for: {}. Focus on risks. Be concise.",
                    user_input
                )
            }
        }
    }

    /// Synthesize the 3 trajectories into a single coherent fusion prompt.
    ///
    /// This is the Haiyue Fusion instruction that guides the final response.
    pub fn synthesize(&self, user_input: &str) -> String {
        format!(
            "User Input: {}\n\
             --- HAIYUE SIMULATION DATA ---\n\
             [OPTIMISTIC (+1)]: {}\n\
             [NEUTRAL (0)]: {}\n\
             [PESSIMISTIC (-1)]: {}\n\
             ------------------------------\n\
             SYNTHESIS INSTRUCTION: Apply 'Haiyue Fusion'.\n\
             1. MITIGATE: Address the risks identified in the Pessimistic trajectory.\n\
             2. ACCELERATE: Capture the value from the Optimistic trajectory.\n\
             3. GROUND: Use the Neutral trajectory for reality checking.\n\
             RESULT: Generate 'The Fastest Coherent Result'. \
             Do not list the trajectories. Output the final consolidated answer directly.",
            user_input, self.optimistic, self.neutral, self.pessimistic
        )
    }
}

// ===========================================================================
// VELOCITY PHYSICS
// ===========================================================================

/// Execution configuration for a given velocity mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityConfig {
    /// Timeout in seconds.
    pub timeout_secs: u64,
    /// Search depth (number of retrieval rounds).
    pub search_depth: u32,
    /// Maximum retries before giving up.
    pub max_retries: u32,
    /// System prompt suffix to inject.
    pub system_suffix: String,
}

/// Velocity Physics — determines execution parameters based on signal.
///
/// Framework 4.B: Turtle vs Hare Protocol.
/// - Layers 1-3 (Surface): HARE VELOCITY (speed/iteration).
/// - Layers 4-6 (Depth): TURTLE INTEGRITY (certainty/verification).
pub struct VelocityPhysics;

impl VelocityPhysics {
    /// Get the execution config for a velocity mode name.
    pub fn config_for(mode: &str) -> VelocityConfig {
        match mode {
            "WAR_SPEED" => VelocityConfig {
                timeout_secs: 10,
                search_depth: 1,
                max_retries: 1,
                system_suffix: "MODE: WAR_SPEED. Output < 60s. No filler. Pure kinetic payload."
                    .into(),
            },
            "TURTLE_INTEGRITY" => VelocityConfig {
                timeout_secs: 30,
                search_depth: 3,
                max_retries: 3,
                system_suffix:
                    "MODE: TURTLE_INTEGRITY. Verify every claim. Build Cathedrals of Logic."
                        .into(),
            },
            "DEEP_RESEARCH" => VelocityConfig {
                timeout_secs: 60,
                search_depth: 5,
                max_retries: 3,
                system_suffix: "MODE: DEEP_RESEARCH. Exhaustive analysis. Cite all sources."
                    .into(),
            },
            "ARCHITECT_PRIME" => VelocityConfig {
                timeout_secs: 45,
                search_depth: 4,
                max_retries: 3,
                system_suffix:
                    "MODE: ARCHITECT_PRIME. High-Entropy Handling. Structure First.".into(),
            },
            _ => VelocityConfig {
                // SYNC_POINT (default)
                timeout_secs: 15,
                search_depth: 2,
                max_retries: 2,
                system_suffix: "MODE: STANDARD. Balanced velocity and integrity.".into(),
            },
        }
    }

    /// Determine velocity mode from entropy and urgency scores.
    pub fn determine_mode(entropy: f64, urgency: f64) -> &'static str {
        if urgency > 0.6 {
            "WAR_SPEED"
        } else if entropy > 0.6 {
            "TURTLE_INTEGRITY"
        } else {
            "SYNC_POINT"
        }
    }

    /// Get config for given signal entropy/urgency.
    pub fn config_from_signal(entropy: f64, urgency: f64) -> VelocityConfig {
        Self::config_for(Self::determine_mode(entropy, urgency))
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- OIS Truth Budget ---

    #[test]
    fn test_ois_initial_budget() {
        let ois = OisTruthBudget::default();
        assert_eq!(ois.budget, 100);
        assert!(ois.check());
        assert_eq!(ois.status(), "SOUND");
    }

    #[test]
    fn test_ois_deduct_by_type() {
        let mut ois = OisTruthBudget::default();
        ois.deduct_by_type(OisCostType::Assumption, "User claim unverified");
        assert_eq!(ois.budget, 80); // 100 - 20
        assert_eq!(ois.history.len(), 1);
    }

    #[test]
    fn test_ois_depleted_halts() {
        let mut ois = OisTruthBudget::new(30);
        ois.deduct_by_type(OisCostType::Undecidable, "Fundamental limit");
        assert!(!ois.check()); // 30 - 40 = -10
        assert!(ois.is_depleted());
        assert_eq!(ois.status(), "DEPLETED");
    }

    #[test]
    fn test_ois_semantic_ledger() {
        let mut ois = OisTruthBudget::new(10);
        assert!(ois.check_ledger(5)); // 10 - 5 = 5
        assert!(ois.check_ledger(5)); // 5 - 5 = 0
        assert!(!ois.check_ledger(5)); // 0 < 5 → refused
    }

    #[test]
    fn test_ois_veto_costs_100() {
        let mut ois = OisTruthBudget::default();
        ois.deduct_by_type(OisCostType::VetoTrigger, "System integrity lock");
        assert_eq!(ois.budget, 0); // 100 - 100 = 0
        assert!(ois.is_depleted());
    }

    // --- Haiyue Microcosm ---

    #[test]
    fn test_haiyue_trajectory_prompts() {
        let p = HaiyueSimulation::trajectory_prompt(Trajectory::Pessimistic, "test input");
        assert!(p.contains("PESSIMISTIC"));
        assert!(p.contains("risks"));
        assert!(p.contains("test input"));
    }

    #[test]
    fn test_haiyue_synthesis() {
        let sim = HaiyueSimulation {
            optimistic: "Growth".into(),
            neutral: "Stable".into(),
            pessimistic: "Decline".into(),
        };
        let result = sim.synthesize("Market analysis");
        assert!(result.contains("HAIYUE SIMULATION DATA"));
        assert!(result.contains("MITIGATE"));
        assert!(result.contains("ACCELERATE"));
        assert!(result.contains("GROUND"));
        assert!(result.contains("Growth"));
        assert!(result.contains("Decline"));
    }

    #[test]
    fn test_haiyue_from_results() {
        let mut results = HashMap::new();
        results.insert(Trajectory::Optimistic, "Best case".into());
        let sim = HaiyueSimulation::from_results(&results);
        assert_eq!(sim.optimistic, "Best case");
        assert_eq!(sim.neutral, "N/A"); // Missing → default
    }

    // --- Velocity Physics ---

    #[test]
    fn test_velocity_war_speed() {
        let mode = VelocityPhysics::determine_mode(0.3, 0.8);
        assert_eq!(mode, "WAR_SPEED");
        let config = VelocityPhysics::config_for(mode);
        assert_eq!(config.timeout_secs, 10);
        assert_eq!(config.max_retries, 1);
    }

    #[test]
    fn test_velocity_turtle_integrity() {
        let mode = VelocityPhysics::determine_mode(0.8, 0.2);
        assert_eq!(mode, "TURTLE_INTEGRITY");
        let config = VelocityPhysics::config_for(mode);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.search_depth, 3);
    }

    #[test]
    fn test_velocity_sync_point_default() {
        let mode = VelocityPhysics::determine_mode(0.3, 0.3);
        assert_eq!(mode, "SYNC_POINT");
        let config = VelocityPhysics::config_for(mode);
        assert_eq!(config.timeout_secs, 15);
    }

    #[test]
    fn test_velocity_config_from_signal() {
        let config = VelocityPhysics::config_from_signal(0.1, 0.9);
        assert_eq!(config.timeout_secs, 10); // WAR_SPEED
    }
}

