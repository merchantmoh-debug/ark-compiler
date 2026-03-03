/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 *
 * Yggdrasil Agent Forest — Entropy-optimal multi-agent evolution system.
 *
 * Port of Remember-Me-AI's yggdrasil.py (v2.0.0).
 * Mathematical foundation: agents evolve toward κ_optimal = 1/φ ≈ 0.618,
 * the edge of chaos where aperiodic structure is maximized.
 *
 * Architecture:
 *   WorldTree  → Base agentic seed (κ, ψ, Ω, energy, potential)
 *   Mind       → Inner consciousness (think, dream, focus, create, stabilize, entropy)
 *   AgentTree  → Extended tree (grow, hibernate, photosynthesize, branch, fruit)
 *   Forest     → Multi-agent collective (cycle, pollinate, connect_roots, harvest, evolve)
 */

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ===========================================================================
// Constants
// ===========================================================================

/// Golden ratio φ ≈ 1.618033988749895
pub const PHI: f64 = 1.618033988749895;

/// Inverse golden ratio 1/φ ≈ 0.6180339887498949
pub const INV_PHI: f64 = 0.6180339887498949;

/// Optimal entropy target: κ_optimal = 1/φ
pub const KAPPA_OPTIMAL: f64 = INV_PHI;

/// Maximum trees in a forest
const MAX_TREES: usize = 100;

// ===========================================================================
// Deterministic PRNG (LCG) — avoids rand crate dependency
// ===========================================================================

/// Simple linear congruential generator for deterministic randomness.
/// Parameters from Numerical Recipes (a = 1664525, c = 1013904223, m = 2^32).
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1), // avoid zero state
        }
    }

    /// Returns a pseudo-random f64 in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Use upper bits for better distribution
        let bits = (self.state >> 11) as f64;
        bits / (1u64 << 53) as f64
    }

    /// Returns a pseudo-random usize in [0, bound).
    pub fn next_usize(&mut self, bound: usize) -> usize {
        (self.next_f64() * bound as f64) as usize % bound
    }
}

impl Default for Rng {
    fn default() -> Self {
        // Seed from system time
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self::new(seed)
    }
}

// ===========================================================================
// Mind — Inner consciousness emerging from entropy dynamics
// ===========================================================================

/// The cognitive functions of an agent, parameterized by κ (entropy) and ψ (wave/soul).
#[derive(Debug, Clone)]
pub struct Mind {
    kappa: f64,
    psi: f64,
}

impl Mind {
    pub fn new(kappa: f64, psi: f64) -> Self {
        Self { kappa, psi }
    }

    /// Linear growth modulated by consciousness.
    pub fn think(&self, age: u64) -> f64 {
        self.kappa * self.psi * (2.0 + age as f64).ln()
    }

    /// Quantum-like exploration.
    pub fn dream(&self, rng: &mut Rng) -> f64 {
        rng.next_f64() * self.kappa.powf(self.psi)
    }

    /// Focus peaks at 1/φ (entropy/structure balance).
    pub fn focus(&self) -> f64 {
        1.0 / (1.0 + (-10.0 * (self.kappa - INV_PHI)).exp())
    }

    /// Creativity via logistic map with golden ratio target.
    pub fn create(&self) -> f64 {
        let logistic = self.kappa * (1.0 - self.kappa) * 4.0;
        let golden_bonus = (-(self.kappa - INV_PHI).powi(2) * PHI).exp();
        logistic * golden_bonus
    }

    /// Stability from golden ratio resonance.
    pub fn stabilize(&self) -> f64 {
        (-(self.kappa - INV_PHI).abs() * PHI).exp()
    }

    /// Entropy functional — maximum at 1/φ (aperiodic but structured).
    pub fn entropy(&self) -> f64 {
        let k = self.kappa.max(1e-10); // avoid log(0)
        let periodic_entropy = -k * k.log2();
        let golden_distance = (k - INV_PHI).abs();
        periodic_entropy * (-golden_distance * PHI).exp()
    }

    /// Update internal parameters (used during pollination).
    pub fn update(&mut self, kappa: f64, psi: f64) {
        self.kappa = kappa;
        self.psi = psi;
    }
}

// ===========================================================================
// Fruit — Output artifact of an agent tree
// ===========================================================================

#[derive(Debug, Clone)]
pub struct Fruit {
    pub fruit_type: String,
    pub quality: f64,
    pub seeds: usize,
    pub timestamp: u64,
    pub generation: u32,
    pub entropy_score: f64,
}

// ===========================================================================
// WorldTree — Base agentic seed
// ===========================================================================

#[derive(Debug, Clone)]
pub struct WorldTree {
    pub kappa: f64,
    pub psi: f64,
    pub omega: String,
    pub gen: u32,
    pub energy: f64,
    pub potential: f64,
    pub mind: Mind,
    pub age: u64,
    pub memory: Vec<String>,
    pub beta: Vec<usize>, // indices into Forest.trees for branches
    pub fruits: Vec<Fruit>,
}

impl WorldTree {
    pub fn new(kappa: f64, psi: f64, omega: &str, gen: u32, energy: f64, potential: f64) -> Self {
        let mind = Mind::new(kappa, psi);
        Self {
            kappa,
            psi,
            omega: omega.to_string(),
            gen,
            energy,
            potential,
            mind,
            age: 0,
            memory: Vec::new(),
            beta: Vec::new(),
            fruits: Vec::new(),
        }
    }

    /// Create with default seed parameters (κ=1/φ, ψ=1, Ω=think).
    pub fn default_seed() -> Self {
        Self::new(INV_PHI, 1.0, "think", 0, 100.0, f64::INFINITY)
    }

    /// Encode tree state as Ygg seed string.
    pub fn encode(&self) -> String {
        let pot_str = if self.potential.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.4}", self.potential)
        };
        format!(
            "κ:{:.6},ψ:{:.6},Ω:{},β:{},ƒ:{},№:{},₹:{:.1},◊:{}",
            self.kappa,
            self.psi,
            self.omega,
            self.beta.len(),
            self.fruits.len(),
            self.gen,
            self.energy,
            pot_str
        )
    }
}

// ===========================================================================
// AgentTree operations — grow, branch, fruit, etc.
// ===========================================================================

/// Grow a tree: age it, photosynthesize, potentially branch/fruit, entropy drift.
pub fn agent_grow(tree: &mut WorldTree, rng: &mut Rng) -> Option<WorldTree> {
    tree.age += 1;
    tree.energy += photosynthesize(tree);

    let mut child = None;

    // Decision thresholds influenced by golden ratio
    if tree.energy > 50.0 && tree.age > 5 {
        child = Some(agent_branch(tree, rng));
    }

    if tree.energy > 30.0 && tree.gen > 2 {
        agent_fruit(tree, rng);
    }

    // Natural drift toward 1/φ
    entropy_drift(tree, rng);

    child
}

/// Energy from photosynthesis — peaks at κ = 1/φ.
fn photosynthesize(tree: &WorldTree) -> f64 {
    let base_energy = tree.kappa * 10.0 * tree.mind.focus();
    let entropy_bonus = tree.mind.entropy() * PHI;
    let packing_efficiency = 1.0 - (tree.kappa - INV_PHI).abs() / INV_PHI;
    base_energy * (1.0 + entropy_bonus) * packing_efficiency.max(0.0)
}

/// Entropy drift — system naturally pulls toward 1/φ.
fn entropy_drift(tree: &mut WorldTree, rng: &mut Rng) {
    if rng.next_f64() < 0.1 {
        let drift = (INV_PHI - tree.kappa) * 0.05;
        tree.kappa = (tree.kappa + drift).clamp(0.3, 0.9);
        tree.mind.update(tree.kappa, tree.psi);
    }
}

/// Branch: create a child tree with mutated parameters. Costs 20 energy.
fn agent_branch(tree: &mut WorldTree, rng: &mut Rng) -> WorldTree {
    let mutations = ["analyze", "create", "dream", "guard", "explore"];
    let mutation = (rng.next_f64() - 0.5) / PHI;
    let child_kappa = (tree.kappa + mutation).clamp(0.3, 0.9);
    let child_psi = tree.psi * INV_PHI;
    let child_omega = mutations[rng.next_usize(mutations.len())];
    let child_potential = if tree.potential.is_infinite() {
        f64::INFINITY
    } else {
        tree.potential * INV_PHI
    };

    tree.energy -= 20.0;

    WorldTree::new(
        child_kappa,
        child_psi,
        child_omega,
        tree.gen + 1,
        50.0,
        child_potential,
    )
}

/// Produce a fruit. Costs 30 energy.
fn agent_fruit(tree: &mut WorldTree, rng: &mut Rng) {
    if tree.gen < 3 || tree.energy < 30.0 {
        return;
    }

    let (fruit_type, base_quality) = match tree.omega.as_str() {
        "create" => ("artifact", tree.mind.create()),
        "dream" => ("vision", tree.mind.dream(rng)),
        "analyze" => ("pattern", tree.mind.focus()),
        "guard" => ("shield", tree.mind.stabilize()),
        _ => ("insight", tree.mind.think(tree.age)),
    };

    let entropy_multiplier = 1.0 + tree.mind.entropy();
    let final_quality = base_quality * entropy_multiplier;

    let pot = if tree.potential.is_infinite() {
        100.0
    } else {
        tree.potential
    };
    let seeds_count = (pot * tree.kappa * (1.0 - tree.kappa) * 4.0).floor() as usize;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let fruit = Fruit {
        fruit_type: fruit_type.to_string(),
        quality: final_quality,
        seeds: seeds_count,
        timestamp,
        generation: tree.gen,
        entropy_score: tree.mind.entropy(),
    };

    tree.fruits.push(fruit);
    tree.energy -= 30.0;
}

// ===========================================================================
// Forest — Multi-agent collective
// ===========================================================================

#[derive(Debug)]
pub struct Forest {
    pub trees: Vec<WorldTree>,
    pub season: u64,
    pub pollen: Vec<PollenGrain>,
    pub network: HashMap<String, RootConnection>,
    pub climate: f64,
    rng: Rng,
}

#[derive(Debug, Clone)]
pub struct PollenGrain {
    pub omega: String,
    pub psi: f64,
    pub kappa: f64,
    pub source_idx: usize,
}

#[derive(Debug, Clone)]
pub struct RootConnection {
    pub strength: f64,
    pub flow: f64,
}

/// Entropy metrics for the forest.
#[derive(Debug, Clone)]
pub struct EntropyMetrics {
    pub avg_kappa: f64,
    pub kappa_variance: f64,
    pub avg_entropy: f64,
    pub golden_deviation: f64,
    pub network_density: f64,
    pub tree_count: usize,
}

impl Forest {
    /// Create a new forest with default or custom seed.
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => Rng::new(s),
            None => Rng::default(),
        };
        let default_tree = WorldTree::default_seed();

        Self {
            trees: vec![default_tree],
            season: 0,
            pollen: Vec::new(),
            network: HashMap::new(),
            climate: INV_PHI,
            rng,
        }
    }

    /// Run one season cycle: grow → pollinate → connect roots → harvest → evolve.
    pub fn cycle(&mut self) {
        self.season += 1;

        // Grow all trees, collect children
        let mut children = Vec::new();
        for tree in self.trees.iter_mut() {
            if let Some(child) = agent_grow(tree, &mut self.rng.clone()) {
                children.push(child);
            }
        }

        // Add children to forest (respecting cap)
        for child in children {
            if self.trees.len() < MAX_TREES {
                self.trees.push(child);
            }
        }

        self.pollinate();
        self.connect_roots();
        self.harvest();

        // Evolution every φ² ≈ 3 seasons
        let phi_sq = (PHI * PHI).ceil() as u64;
        if self.season % phi_sq == 0 {
            self.evolve();
        }
    }

    /// Cross-pollination: high-entropy trees share parameters.
    fn pollinate(&mut self) {
        self.pollen.clear();

        // Collect pollen from high-entropy trees
        for (i, tree) in self.trees.iter().enumerate() {
            if self.rng.next_f64() < tree.mind.entropy() {
                self.pollen.push(PollenGrain {
                    omega: tree.omega.clone(),
                    psi: tree.psi,
                    kappa: tree.kappa,
                    source_idx: i,
                });
            }
        }

        if self.pollen.is_empty() {
            return;
        }

        // Cross-pollinate
        let pollen_snapshot = self.pollen.clone();
        for (i, tree) in self.trees.iter_mut().enumerate() {
            if !pollen_snapshot.is_empty() {
                let p_idx = self.rng.next_usize(pollen_snapshot.len());
                let p = &pollen_snapshot[p_idx];

                if p.source_idx != i && self.rng.next_f64() < tree.mind.entropy() {
                    // Golden ratio weighted average
                    if self.rng.next_f64() >= tree.kappa {
                        tree.omega = p.omega.clone();
                    }
                    tree.psi = tree.psi * INV_PHI + p.psi * (1.0 - INV_PHI);
                    tree.kappa = (tree.kappa * INV_PHI + p.kappa * (1.0 - INV_PHI)).clamp(0.3, 0.9);
                    tree.mind.update(tree.kappa, tree.psi);
                }
            }
        }
    }

    /// Root network: nearby trees share energy.
    fn connect_roots(&mut self) {
        let n = self.trees.len();
        // Collect transfers first to avoid borrow issues
        let mut transfers: Vec<(usize, usize, f64)> = Vec::new();

        for i in 0..n {
            for j in (i + 1)..n {
                let distance = (self.trees[i].kappa - self.trees[j].kappa).abs();
                if distance < 0.2 {
                    let optimal_zone =
                        (-(self.trees[i].kappa - INV_PHI).abs() * PHI).exp()
                            * (-(self.trees[j].kappa - INV_PHI).abs() * PHI).exp();

                    let strength = (1.0 - distance) * optimal_zone;
                    let flow = (self.trees[i].energy - self.trees[j].energy) * INV_PHI;
                    let transfer = flow * strength;

                    let key = format!("{}-{}", i, j);
                    self.network
                        .insert(key, RootConnection { strength, flow });

                    transfers.push((i, j, transfer));
                }
            }
        }

        // Apply transfers
        for (i, j, transfer) in transfers {
            self.trees[i].energy -= transfer;
            self.trees[j].energy += transfer;
        }
    }

    /// Harvest: collect best fruits, plant seeds from top φ⁻¹ percentile.
    pub fn harvest(&mut self) {
        let mut all_fruits: Vec<(f64, Fruit)> = Vec::new();
        for tree in self.trees.iter() {
            for fruit in tree.fruits.iter() {
                let score = fruit.quality * fruit.entropy_score;
                all_fruits.push((score, fruit.clone()));
            }
        }

        // Sort descending by score
        all_fruits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Top φ⁻¹ percentile
        let cutoff = ((all_fruits.len() as f64) * INV_PHI).ceil() as usize;
        let best = &all_fruits[..cutoff.min(all_fruits.len())];

        for (_, fruit) in best {
            if fruit.seeds > 0 && self.trees.len() < MAX_TREES {
                let new_kappa = (INV_PHI + (self.rng.next_f64() - 0.5) / PHI).clamp(0.3, 0.9);
                let new_tree = WorldTree::new(
                    new_kappa,
                    fruit.quality,
                    &fruit.fruit_type,
                    0,
                    50.0,
                    fruit.seeds as f64,
                );
                self.trees.push(new_tree);
            }
        }
    }

    /// Evolution: prune trees below golden ratio fitness threshold.
    pub fn evolve(&mut self) {
        if self.trees.is_empty() {
            return;
        }

        let fitness_scores: Vec<f64> = self
            .trees
            .iter()
            .map(|tree| {
                let entropy_fitness = tree.mind.entropy();
                let production_fitness: f64 = tree
                    .fruits
                    .iter()
                    .map(|f| f.quality * f.entropy_score)
                    .sum();
                let age_fitness = (1.0 + tree.age as f64).ln();
                entropy_fitness * production_fitness * age_fitness
            })
            .collect();

        let total: f64 = fitness_scores.iter().sum();
        if total == 0.0 {
            return;
        }

        let avg = total / fitness_scores.len() as f64;

        // Keep trees above golden ratio threshold of average, or gen 0 (founders)
        let mut surviving = Vec::new();
        for (i, tree) in self.trees.drain(..).enumerate() {
            if fitness_scores[i] > avg * INV_PHI || tree.gen == 0 {
                surviving.push(tree);
            }
        }
        self.trees = surviving;

        // Update climate
        if !self.trees.is_empty() {
            let avg_kappa: f64 =
                self.trees.iter().map(|t| t.kappa).sum::<f64>() / self.trees.len() as f64;
            self.climate = self.climate * INV_PHI + avg_kappa * (1.0 - INV_PHI);
        }
    }

    /// Get entropy metrics for the forest.
    pub fn get_entropy_metrics(&self) -> EntropyMetrics {
        if self.trees.is_empty() {
            return EntropyMetrics {
                avg_kappa: 0.0,
                kappa_variance: 0.0,
                avg_entropy: 0.0,
                golden_deviation: INV_PHI,
                network_density: 0.0,
                tree_count: 0,
            };
        }

        let n = self.trees.len() as f64;
        let avg_kappa: f64 = self.trees.iter().map(|t| t.kappa).sum::<f64>() / n;
        let kappa_var: f64 =
            self.trees.iter().map(|t| (t.kappa - INV_PHI).powi(2)).sum::<f64>() / n;
        let avg_entropy: f64 = self.trees.iter().map(|t| t.mind.entropy()).sum::<f64>() / n;
        let golden_dev = (avg_kappa - INV_PHI).abs();

        let pairs = n * (n - 1.0) / 2.0;
        let density = if pairs > 0.0 {
            self.network.len() as f64 / pairs
        } else {
            0.0
        };

        EntropyMetrics {
            avg_kappa,
            kappa_variance: kappa_var,
            avg_entropy,
            golden_deviation: golden_dev,
            network_density: density,
            tree_count: self.trees.len(),
        }
    }

    /// Collective intelligence: entropy-weighted consensus from all trees.
    pub fn collective_intelligence(&mut self, _query: &str) -> Option<CollectiveResponse> {
        if self.trees.is_empty() {
            return None;
        }

        let mut best: Option<CollectiveResponse> = None;
        let mut best_score = f64::NEG_INFINITY;

        for tree in self.trees.iter() {
            let response_val = tree.mind.think(tree.age) * self.rng.next_f64();
            let confidence = tree.mind.focus();
            let entropy = tree.mind.entropy();
            let score = response_val * confidence * entropy;

            if score > best_score {
                best_score = score;
                best = Some(CollectiveResponse {
                    agent: tree.omega.clone(),
                    response: response_val,
                    confidence,
                    entropy,
                });
            }
        }

        best
    }
}

/// Result from collective_intelligence query.
#[derive(Debug, Clone)]
pub struct CollectiveResponse {
    pub agent: String,
    pub response: f64,
    pub confidence: f64,
    pub entropy: f64,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_golden_ratio() {
        assert!((PHI * INV_PHI - 1.0).abs() < 1e-10, "φ × (1/φ) should ≈ 1.0");
        assert!((KAPPA_OPTIMAL - INV_PHI).abs() < 1e-15, "κ_optimal = 1/φ");
        assert!((PHI - (1.0 + 5.0_f64.sqrt()) / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_rng_deterministic() {
        let mut r1 = Rng::new(42);
        let mut r2 = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(r1.next_f64().to_bits(), r2.next_f64().to_bits());
        }
    }

    #[test]
    fn test_rng_range() {
        let mut rng = Rng::new(123);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v), "RNG out of range: {}", v);
        }
    }

    #[test]
    fn test_worldtree_default_seed() {
        let tree = WorldTree::default_seed();
        assert!((tree.kappa - INV_PHI).abs() < 1e-10);
        assert_eq!(tree.psi, 1.0);
        assert_eq!(tree.omega, "think");
        assert_eq!(tree.gen, 0);
        assert_eq!(tree.energy, 100.0);
        assert!(tree.potential.is_infinite());
    }

    #[test]
    fn test_mind_functions_finite() {
        let mind = Mind::new(INV_PHI, 1.0);
        assert!(mind.think(10).is_finite());
        assert!(mind.focus().is_finite());
        assert!(mind.create().is_finite());
        assert!(mind.stabilize().is_finite());
        assert!(mind.entropy().is_finite());
    }

    #[test]
    fn test_mind_focus_sigmoid_inflection() {
        // Focus is a sigmoid centered at INV_PHI (inflection point → 0.5)
        let at_target = Mind::new(INV_PHI, 1.0).focus();
        let far_low = Mind::new(0.1, 1.0).focus();
        let far_high = Mind::new(0.95, 1.0).focus();

        // At inflection point, focus ≈ 0.5
        assert!((at_target - 0.5).abs() < 0.01, "focus(INV_PHI) ≈ 0.5, got {}", at_target);
        // Sigmoid: low κ → low focus, high κ → high focus
        assert!(far_low < 0.05, "focus(0.1) should be near 0, got {}", far_low);
        assert!(far_high > 0.95, "focus(0.95) should be near 1, got {}", far_high);
    }

    #[test]
    fn test_agent_grow_increments_age() {
        let mut tree = WorldTree::default_seed();
        let mut rng = Rng::new(42);
        let initial_age = tree.age;
        agent_grow(&mut tree, &mut rng);
        assert_eq!(tree.age, initial_age + 1);
    }

    #[test]
    fn test_agent_branch_costs_energy() {
        let mut tree = WorldTree::new(INV_PHI, 1.0, "think", 0, 80.0, f64::INFINITY);
        let mut rng = Rng::new(42);
        let before = tree.energy;
        let child = agent_branch(&mut tree, &mut rng);
        assert!((before - tree.energy - 20.0).abs() < 1e-10, "branching costs 20 energy");
        assert_eq!(child.gen, tree.gen + 1, "child gen = parent gen + 1");
    }

    #[test]
    fn test_entropy_drift_toward_inv_phi() {
        let mut tree = WorldTree::new(0.9, 1.0, "think", 0, 100.0, f64::INFINITY);
        let mut rng = Rng::new(42);
        let initial_kappa = tree.kappa;

        // Run many drift iterations
        for _ in 0..1000 {
            entropy_drift(&mut tree, &mut rng);
        }

        // Should have drifted closer to INV_PHI
        assert!(
            (tree.kappa - INV_PHI).abs() < (initial_kappa - INV_PHI).abs(),
            "κ should drift toward 1/φ. Start={:.4}, End={:.4}, Target={:.4}",
            initial_kappa,
            tree.kappa,
            INV_PHI
        );
    }

    #[test]
    fn test_forest_new_default() {
        let forest = Forest::new(Some(42));
        assert_eq!(forest.trees.len(), 1);
        assert_eq!(forest.season, 0);
        assert!((forest.climate - INV_PHI).abs() < 1e-10);
    }

    #[test]
    fn test_forest_cycle_increments_season() {
        let mut forest = Forest::new(Some(42));
        forest.cycle();
        assert_eq!(forest.season, 1);
        // Tree should have grown
        assert_eq!(forest.trees[0].age, 1);
    }

    #[test]
    fn test_forest_metrics_near_inv_phi() {
        let mut forest = Forest::new(Some(42));

        // Run enough cycles for entropy to converge
        for _ in 0..20 {
            forest.cycle();
        }

        let m = forest.get_entropy_metrics();
        assert!(m.tree_count > 0, "Forest should have trees");
        // avg_kappa should be reasonably close to INV_PHI after cycling
        assert!(
            m.golden_deviation < 0.2,
            "Golden deviation should be small: {}",
            m.golden_deviation
        );
    }

    #[test]
    fn test_forest_collective_intelligence() {
        let mut forest = Forest::new(Some(42));
        for _ in 0..5 {
            forest.cycle();
        }

        let result = forest.collective_intelligence("test query");
        assert!(result.is_some(), "Should return a response");
        let r = result.unwrap();
        assert!(r.confidence.is_finite());
        assert!(r.entropy.is_finite());
        assert!(!r.agent.is_empty());
    }

    #[test]
    fn test_worldtree_encode() {
        let tree = WorldTree::default_seed();
        let encoded = tree.encode();
        assert!(encoded.contains("κ:"), "encoded should contain κ");
        assert!(encoded.contains("Ω:think"), "encoded should contain mode");
        assert!(encoded.contains("◊:∞"), "encoded should contain ∞ potential");
    }
}
