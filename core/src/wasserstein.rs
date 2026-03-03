/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 *
 * Wasserstein Metric — Entropy-Regularized Optimal Transport (Sinkhorn)
 *
 * Ported from Remember-Me-AI's math/transport.py.
 *
 * Unlike cosine similarity (which measures angles), Wasserstein measures the
 * "Work" required to transform the Memory Distribution into the Query
 * Distribution. This quantifies the "Information Mass" of a memory fragment.
 *
 * Pure Rust implementation — no PyTorch/BLAS dependency.
 * Matrices are small (N <= 50, D <= 384) so native ops are fast enough.
 */

use serde::{Deserialize, Serialize};

// ===========================================================================
// Dense Vector / Matrix helpers (row-major, heap-allocated)
// ===========================================================================

/// A dense row-major matrix stored as a flat `Vec<f64>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseMat {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl DenseMat {
    /// Create a zero matrix of shape `[rows, cols]`.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    /// Get element at `(i, j)`.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i * self.cols + j]
    }

    /// Set element at `(i, j)`.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, val: f64) {
        self.data[i * self.cols + j] = val;
    }

    /// Sum of all elements in a specific row.
    pub fn row_sum(&self, i: usize) -> f64 {
        let start = i * self.cols;
        self.data[start..start + self.cols].iter().sum()
    }

    /// Index of the maximum element in the entire matrix.
    pub fn argmax(&self) -> usize {
        self.data
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Argmax returning (row, col).
    pub fn argmax_rc(&self) -> (usize, usize) {
        let idx = self.argmax();
        (idx / self.cols, idx % self.cols)
    }

    /// In-place division by scalar.
    pub fn div_scalar(&mut self, s: f64) {
        for v in &mut self.data {
            *v /= s;
        }
    }

    /// In-place clamp to `[lo, hi]`.
    pub fn clamp(&mut self, lo: f64, hi: f64) {
        for v in &mut self.data {
            *v = v.clamp(lo, hi);
        }
    }
}

/// A dense 1-D vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseVec {
    pub data: Vec<f64>,
}

impl DenseVec {
    pub fn zeros(len: usize) -> Self {
        Self {
            data: vec![0.0; len],
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Softmax in-place, returns self for chaining.
    pub fn softmax(&mut self) -> &Self {
        if self.data.is_empty() {
            return self;
        }
        let max = self.data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut sum = 0.0;
        for v in &mut self.data {
            *v = (*v - max).exp();
            sum += *v;
        }
        if sum > 0.0 {
            for v in &mut self.data {
                *v /= sum;
            }
        }
        self
    }
}

// ===========================================================================
// Wasserstein Metric
// ===========================================================================

/// Configuration for the Wasserstein optimal transport metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WassersteinConfig {
    /// Entropic regularization parameter (smoothness). Default: 0.1.
    pub epsilon: f64,
    /// Maximum Sinkhorn iterations. Default: 100.
    pub max_iter: usize,
    /// Convergence tolerance. Default: 1e-6.
    pub tol: f64,
}

impl Default for WassersteinConfig {
    fn default() -> Self {
        Self {
            epsilon: 0.1,
            max_iter: 100,
            tol: 1e-6,
        }
    }
}

/// Wasserstein Optimal Transport metric.
///
/// Computes the "information mass" of memory fragments relative to
/// a query state using entropy-regularized optimal transport (Sinkhorn).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WassersteinMetric {
    pub config: WassersteinConfig,
}

impl Default for WassersteinMetric {
    fn default() -> Self {
        Self::new()
    }
}

impl WassersteinMetric {
    /// Create with default config.
    pub fn new() -> Self {
        Self {
            config: WassersteinConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: WassersteinConfig) -> Self {
        Self { config }
    }

    /// Compute squared Euclidean distance cost matrix.
    ///
    /// `x`: `[N, D]` row-major, `y`: `[M, D]` row-major.
    /// Returns cost matrix `C` of shape `[N, M]` where `C[i][j] = ||x_i - y_j||^2`.
    ///
    /// Optionally accepts pre-computed squared norms for `x` (length N) and `y` (length M).
    pub fn compute_cost_matrix(
        &self,
        x: &[f64],
        x_rows: usize,
        y: &[f64],
        y_rows: usize,
        dim: usize,
        x_norms: Option<&[f64]>,
        y_norms: Option<&[f64]>,
    ) -> DenseMat {
        let n = x_rows;
        let m = y_rows;

        // Compute ||x_i||^2 for each row
        let x_sq: Vec<f64> = match x_norms {
            Some(norms) => norms.to_vec(),
            None => (0..n)
                .map(|i| {
                    let row = &x[i * dim..(i + 1) * dim];
                    row.iter().map(|v| v * v).sum()
                })
                .collect(),
        };

        // Compute ||y_j||^2 for each row
        let y_sq: Vec<f64> = match y_norms {
            Some(norms) => norms.to_vec(),
            None => (0..m)
                .map(|j| {
                    let row = &y[j * dim..(j + 1) * dim];
                    row.iter().map(|v| v * v).sum()
                })
                .collect(),
        };

        // C[i][j] = ||x_i||^2 + ||y_j||^2 - 2 * <x_i, y_j>
        let mut cost = DenseMat::zeros(n, m);
        for i in 0..n {
            let x_row = &x[i * dim..(i + 1) * dim];
            for j in 0..m {
                let y_row = &y[j * dim..(j + 1) * dim];
                let dot: f64 = x_row.iter().zip(y_row.iter()).map(|(a, b)| a * b).sum();
                let c = (x_sq[i] + y_sq[j] - 2.0 * dot).max(0.0);
                cost.set(i, j, c);
            }
        }
        cost
    }

    /// Compute transport mass scores for each memory in `memory_bank`
    /// relative to `query_state`.
    ///
    /// - `query_state`: `[1, D]` — the current coherent state vector.
    /// - `memory_bank`: `[N, D]` — buffer of memory vectors.
    /// - Returns: `DenseVec` of length N with relevance scores (sum ≈ 1.0).
    ///
    /// **Optimization**: When `M = 1` (single query), Sinkhorn degenerates to
    /// softmax — 5× faster and avoids numerical underflow.
    pub fn compute_transport_mass(
        &self,
        query_state: &[f64],
        memory_bank: &[f64],
        n_memories: usize,
        dim: usize,
        memory_norms: Option<&[f64]>,
    ) -> DenseVec {
        if n_memories == 0 {
            return DenseVec::zeros(0);
        }

        // Cost matrix C: [N, 1]
        let cost = self.compute_cost_matrix(
            memory_bank,
            n_memories,
            query_state,
            1,
            dim,
            memory_norms,
            None,
        );

        // M = 1 → Softmax fast path
        // Mass ~ exp(-C / epsilon), normalized
        let mut logits = DenseVec {
            data: (0..n_memories)
                .map(|i| -cost.get(i, 0) / self.config.epsilon)
                .collect(),
        };
        logits.softmax();
        logits
    }

    /// Find the index of the memory with **minimum** information mass
    /// (i.e., the one to evict). This is the argmax of the cost matrix
    /// since mass ~ exp(-cost).
    pub fn find_eviction_target(
        &self,
        query_state: &[f64],
        memory_bank: &[f64],
        n_memories: usize,
        dim: usize,
        memory_norms: Option<&[f64]>,
    ) -> usize {
        let cost = self.compute_cost_matrix(
            memory_bank,
            n_memories,
            query_state,
            1,
            dim,
            memory_norms,
            None,
        );

        // Max cost = min mass = eviction target
        let (row, _) = cost.argmax_rc();
        row
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: create a simple vector
    fn vec_f64(vals: &[f64]) -> Vec<f64> {
        vals.to_vec()
    }

    #[test]
    fn test_cost_matrix_identity() {
        let w = WassersteinMetric::new();
        // Same point → zero cost
        let x = vec_f64(&[1.0, 0.0, 0.0]);
        let cost = w.compute_cost_matrix(&x, 1, &x, 1, 3, None, None);
        assert!(
            (cost.get(0, 0)).abs() < 1e-10,
            "Same point should have zero cost"
        );
    }

    #[test]
    fn test_cost_matrix_symmetric() {
        let w = WassersteinMetric::new();
        let x = vec_f64(&[1.0, 2.0]);
        let y = vec_f64(&[3.0, 4.0]);
        let c1 = w.compute_cost_matrix(&x, 1, &y, 1, 2, None, None);
        let c2 = w.compute_cost_matrix(&y, 1, &x, 1, 2, None, None);
        assert!(
            (c1.get(0, 0) - c2.get(0, 0)).abs() < 1e-10,
            "Cost should be symmetric"
        );
    }

    #[test]
    fn test_cost_matrix_known_value() {
        let w = WassersteinMetric::new();
        // ||[1,0] - [0,1]||^2 = 1 + 1 = 2
        let x = vec_f64(&[1.0, 0.0]);
        let y = vec_f64(&[0.0, 1.0]);
        let cost = w.compute_cost_matrix(&x, 1, &y, 1, 2, None, None);
        assert!(
            (cost.get(0, 0) - 2.0).abs() < 1e-10,
            "Expected cost 2.0, got {}",
            cost.get(0, 0)
        );
    }

    #[test]
    fn test_transport_mass_sums_to_one() {
        let w = WassersteinMetric::new();
        // 3 memories, dim=2
        let bank = vec_f64(&[1.0, 0.0, 0.0, 1.0, 0.5, 0.5]);
        let query = vec_f64(&[0.5, 0.5]);
        let mass = w.compute_transport_mass(&query, &bank, 3, 2, None);
        let total: f64 = mass.data.iter().sum();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "Mass should sum to 1.0, got {}",
            total
        );
    }

    #[test]
    fn test_closest_memory_has_highest_mass() {
        let w = WassersteinMetric::new();
        // Memory 2 is closest to query
        let bank = vec_f64(&[
            10.0, 10.0, // far
            -5.0, -5.0, // far
            1.1, 1.1, // close
        ]);
        let query = vec_f64(&[1.0, 1.0]);
        let mass = w.compute_transport_mass(&query, &bank, 3, 2, None);
        assert!(
            mass.data[2] > mass.data[0] && mass.data[2] > mass.data[1],
            "Closest memory should have highest mass: {:?}",
            mass.data
        );
    }

    #[test]
    fn test_eviction_target_is_farthest() {
        let w = WassersteinMetric::new();
        // Memory 0 is farthest from query
        let bank = vec_f64(&[
            100.0, 100.0, // far → eviction target
            1.0, 1.0, // close
            0.9, 0.9, // close
        ]);
        let query = vec_f64(&[1.0, 1.0]);
        let target = w.find_eviction_target(&query, &bank, 3, 2, None);
        assert_eq!(target, 0, "Farthest memory should be eviction target");
    }

    #[test]
    fn test_empty_bank() {
        let w = WassersteinMetric::new();
        let query = vec_f64(&[1.0, 1.0]);
        let mass = w.compute_transport_mass(&query, &[], 0, 2, None);
        assert!(mass.is_empty());
    }

    #[test]
    fn test_single_memory() {
        let w = WassersteinMetric::new();
        let bank = vec_f64(&[1.0, 0.0]);
        let query = vec_f64(&[0.0, 1.0]);
        let mass = w.compute_transport_mass(&query, &bank, 1, 2, None);
        assert_eq!(mass.len(), 1);
        assert!(
            (mass.data[0] - 1.0).abs() < 1e-6,
            "Single memory should have mass 1.0"
        );
    }

    #[test]
    fn test_precomputed_norms() {
        let w = WassersteinMetric::new();
        let bank = vec_f64(&[1.0, 0.0, 0.0, 1.0]);
        let query = vec_f64(&[0.5, 0.5]);

        // Pre-compute norms: ||[1,0]||^2=1, ||[0,1]||^2=1
        let norms = vec_f64(&[1.0, 1.0]);

        let m1 = w.compute_transport_mass(&query, &bank, 2, 2, None);
        let m2 = w.compute_transport_mass(&query, &bank, 2, 2, Some(&norms));

        for i in 0..2 {
            assert!(
                (m1.data[i] - m2.data[i]).abs() < 1e-10,
                "Pre-computed norms should give same result"
            );
        }
    }

    #[test]
    fn test_softmax_numerical_stability() {
        // Large values should not overflow
        let mut v = DenseVec {
            data: vec![1000.0, 1001.0, 999.0],
        };
        v.softmax();
        let total: f64 = v.data.iter().sum();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "Softmax should sum to 1.0 even with large values"
        );
    }
}
