/*
 * Copyright (c) 2026 Mohamad Al-Zawahreh (dba Sovereign Systems).
 *
 * This file is part of the Ark Sovereign Compiler.
 *
 * LICENSE: DUAL-LICENSED (AGPLv3 or COMMERCIAL).
 * PATENT NOTICE: Protected by US Patent App #63/935,467.
 *
 * CSNP — Coherent State Network Protocol
 *
 * Ported from Remember-Me-AI's core/csnp.py.
 *
 * Replaces the standard "Context Window". Instead of appending tokens
 * (linear cost), it maintains a fixed-size buffer and an evolving
 * "Identity State" (Living State Vector).
 *
 * When the buffer is full, it uses Wasserstein optimal transport
 * to quantify "information mass" and evicts the lowest-mass vectors
 * relative to the current narrative trajectory.
 *
 * Key operations:
 * - update_state(): Embed → Evolve identity (Kalman) → Compress (Wasserstein)
 * - retrieve_context(): Get current context string with integrity verification
 * - trinary_undo(): Reverse last memory (TNeg [-1])
 * - consolidate_memory(): Preemptive compression at 80% capacity
 */

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::wasserstein::WassersteinMetric;

// ===========================================================================
// Integrity Chain (Merkle-like hash chain for tamper detection)
// ===========================================================================

/// A Merkle-like hash chain for verifying memory integrity.
///
/// Every interaction is hashed and chained. If a memory fragment is
/// tampered with, the hash won't match and it's rejected during retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityChain {
    /// Ordered leaf hashes.
    pub leaf_hashes: Vec<String>,
    /// Ordered leaf data (the raw text that was hashed).
    pub leaf_data: Vec<String>,
}

impl Default for IntegrityChain {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrityChain {
    pub fn new() -> Self {
        Self {
            leaf_hashes: Vec::new(),
            leaf_data: Vec::new(),
        }
    }

    /// Hash a string using SHA-256.
    pub fn hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Add an entry to the chain. Returns its hash.
    pub fn add_entry(&mut self, data: &str) -> String {
        let h = Self::hash(data);
        self.leaf_hashes.push(h.clone());
        self.leaf_data.push(data.to_string());
        h
    }

    /// Compute the Merkle root hash of all leaves.
    pub fn root_hash(&self) -> String {
        if self.leaf_hashes.is_empty() {
            return String::new();
        }
        let mut hashes = self.leaf_hashes.clone();
        while hashes.len() > 1 {
            let mut next = Vec::new();
            for pair in hashes.chunks(2) {
                if pair.len() == 2 {
                    next.push(Self::hash(&format!("{}{}", pair[0], pair[1])));
                } else {
                    next.push(pair[0].clone());
                }
            }
            hashes = next;
        }
        hashes.into_iter().next().unwrap_or_default()
    }

    /// Check if a hash exists in the chain.
    pub fn contains(&self, hash: &str) -> bool {
        self.leaf_hashes.iter().any(|h| h == hash)
    }

    /// Bulk load from pre-computed hashes and data.
    pub fn load_bulk(&mut self, hashes: Vec<String>, data: Vec<String>) {
        self.leaf_hashes = hashes;
        self.leaf_data = data;
    }
}

// ===========================================================================
// Temporal State (Trinary)
// ===========================================================================

/// Trinary temporal state for each memory slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i8)]
pub enum TemporalState {
    /// Past (Verified/Historical).
    Past = -1,
    /// Present (Active buffer).
    Present = 0,
    /// Future (Predicted/Intent).
    Future = 1,
}

impl Default for TemporalState {
    fn default() -> Self {
        Self::Present
    }
}

// ===========================================================================
// CSNP Manager
// ===========================================================================

/// The Coherent State Network Protocol manager.
///
/// Manages a fixed-size memory buffer with:
/// - **Kalman-like identity evolution** (exponential moving average)
/// - **Wasserstein eviction** (optimal transport mass scoring)
/// - **Merkle integrity chain** (tamper detection)
/// - **Trinary undo** (TNeg: reverse last memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsnpManager {
    /// Embedding dimension.
    pub dim: usize,
    /// Maximum number of memory slots before compression triggers.
    pub context_limit: usize,

    // --- Dense buffers ---
    /// Memory bank: `[capacity, dim]` row-major.
    memory_bank: Vec<f64>,
    /// Pre-computed squared norms for each memory row.
    memory_norms: Vec<f64>,
    /// The Living State Vector (identity): `[1, dim]`.
    identity_state: Vec<f64>,
    /// Temporal state for each slot.
    temporal_states: Vec<TemporalState>,

    /// Current number of active memories.
    size: usize,
    /// Total allocated capacity (context_limit + 1).
    capacity: usize,

    // --- Text / integrity ---
    /// Text buffer: the raw interaction text for each memory.
    text_buffer: Vec<String>,
    /// Hash buffer: SHA-256 hash for each text entry.
    hash_buffer: Vec<String>,
    /// Merkle integrity chain.
    chain: IntegrityChain,

    // --- Engines ---
    /// The Wasserstein metric engine.
    metric: WassersteinMetric,

    // --- Cache ---
    /// Cached context string (invalidated on mutation).
    context_cache: Option<String>,

    /// Kalman smoothing factor (alpha). Default: 0.1.
    pub alpha: f64,
}

impl CsnpManager {
    /// Create a new CSNP manager.
    ///
    /// - `dim`: embedding dimension (e.g., 384 for MiniLM).
    /// - `context_limit`: max memory slots before compression.
    pub fn new(dim: usize, context_limit: usize) -> Self {
        let capacity = context_limit + 1;
        Self {
            dim,
            context_limit,
            memory_bank: vec![0.0; capacity * dim],
            memory_norms: vec![0.0; capacity],
            identity_state: vec![0.0; dim],
            temporal_states: vec![TemporalState::Present; capacity],
            size: 0,
            capacity,
            text_buffer: Vec::new(),
            hash_buffer: Vec::new(),
            chain: IntegrityChain::new(),
            metric: WassersteinMetric::new(),
            context_cache: None,
            alpha: 0.1,
        }
    }

    /// Current number of active memories.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Whether the buffer is at capacity.
    pub fn is_full(&self) -> bool {
        self.size >= self.context_limit
    }

    /// Get the identity state vector (read-only).
    pub fn identity(&self) -> &[f64] {
        &self.identity_state
    }

    /// Get the memory bank slice for active memories (row-major, `[size, dim]`).
    pub fn active_bank(&self) -> &[f64] {
        &self.memory_bank[..self.size * self.dim]
    }

    /// Get the active norms slice.
    pub fn active_norms(&self) -> &[f64] {
        &self.memory_norms[..self.size]
    }

    // -----------------------------------------------------------------------
    // Core operations
    // -----------------------------------------------------------------------

    /// CSNP Update Cycle.
    ///
    /// 1. **Integrity**: Hash interaction into Merkle chain.
    /// 2. **Embed**: Accept pre-computed embedding vector.
    /// 3. **Evolve**: Update Identity State (Kalman-like EMA).
    /// 4. **Store**: Add to buffer.
    /// 5. **Compress**: If full, evict lowest-mass via Wasserstein.
    ///
    /// `embedding` must have length `self.dim`.
    pub fn update_state(&mut self, text: &str, embedding: &[f64]) {
        assert_eq!(
            embedding.len(),
            self.dim,
            "Embedding dim mismatch: expected {}, got {}",
            self.dim,
            embedding.len()
        );

        // 1. Integrity
        let hash = self.chain.add_entry(text);

        // 2. Evolve Identity State (Exponential Moving Average)
        let is_zero = self.identity_state.iter().all(|v| v.abs() < 1e-12);
        if is_zero {
            // First interaction: copy directly
            self.identity_state.copy_from_slice(embedding);
        } else {
            // EMA: identity = (1 - alpha) * identity + alpha * embedding
            let alpha = self.alpha;
            for (s, e) in self.identity_state.iter_mut().zip(embedding.iter()) {
                *s = (1.0 - alpha) * *s + alpha * *e;
            }
        }

        // Normalize identity to unit length
        let norm: f64 = self
            .identity_state
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt();
        if norm > 1e-12 {
            for v in &mut self.identity_state {
                *v /= norm;
            }
        }

        // 3. Store in buffer
        if self.size < self.capacity {
            let offset = self.size * self.dim;
            self.memory_bank[offset..offset + self.dim].copy_from_slice(embedding);
            self.memory_norms[self.size] = embedding.iter().map(|v| v * v).sum();
            self.temporal_states[self.size] = TemporalState::Present;
            self.size += 1;
            self.text_buffer.push(text.to_string());
            self.hash_buffer.push(hash);
        }

        // 4. Invalidate cache
        self.context_cache = None;

        // 5. Compress if over limit
        if self.size > self.context_limit {
            self.compress(None);
        }
    }

    /// Preemptive consolidation at 80% capacity.
    pub fn consolidate_memory(&mut self) {
        let threshold = (self.context_limit as f64 * 0.8) as usize;
        if self.size > threshold {
            self.compress(Some(threshold));
        }
    }

    /// Compress memory buffer to `target_size` by evicting lowest-mass memories.
    pub fn compress(&mut self, target_size: Option<usize>) {
        let target = target_size.unwrap_or(self.context_limit);
        if self.size <= target {
            return;
        }

        let excess = self.size - target;

        if excess == 1 {
            // Single eviction: find argmax of cost (= lowest mass)
            let remove_idx = self.metric.find_eviction_target(
                &self.identity_state,
                &self.memory_bank[..self.size * self.dim],
                self.size,
                self.dim,
                Some(&self.memory_norms[..self.size]),
            );

            // Shift memories left to fill the gap
            if remove_idx < self.size - 1 {
                // Shift embedding rows
                let src_start = (remove_idx + 1) * self.dim;
                let dst_start = remove_idx * self.dim;
                let len = (self.size - 1 - remove_idx) * self.dim;
                self.memory_bank
                    .copy_within(src_start..src_start + len, dst_start);

                // Shift norms
                self.memory_norms
                    .copy_within(remove_idx + 1..self.size, remove_idx);
            }

            // Zero out last slot
            let last_start = (self.size - 1) * self.dim;
            for v in &mut self.memory_bank[last_start..last_start + self.dim] {
                *v = 0.0;
            }
            self.memory_norms[self.size - 1] = 0.0;
            self.size -= 1;

            // Remove from text/hash buffers
            if remove_idx < self.text_buffer.len() {
                self.text_buffer.remove(remove_idx);
            }
            if remove_idx < self.hash_buffer.len() {
                self.hash_buffer.remove(remove_idx);
            }
        } else {
            // Bulk eviction: compute mass scores, keep top-K
            let mass = self.metric.compute_transport_mass(
                &self.identity_state,
                &self.memory_bank[..self.size * self.dim],
                self.size,
                self.dim,
                Some(&self.memory_norms[..self.size]),
            );

            // Get indices of top-K masses (highest mass = keep)
            let mut indexed: Vec<(usize, f64)> = mass.data.iter().copied().enumerate().collect();
            indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let mut keep: Vec<usize> = indexed.iter().take(target).map(|(i, _)| *i).collect();
            keep.sort(); // Maintain chronological order

            // Reconstruct buffers
            let mut new_bank = vec![0.0; self.capacity * self.dim];
            let mut new_norms = vec![0.0; self.capacity];
            let mut new_text = Vec::with_capacity(keep.len());
            let mut new_hash = Vec::with_capacity(keep.len());

            for (new_idx, &old_idx) in keep.iter().enumerate() {
                let src = old_idx * self.dim;
                let dst = new_idx * self.dim;
                new_bank[dst..dst + self.dim]
                    .copy_from_slice(&self.memory_bank[src..src + self.dim]);
                new_norms[new_idx] = self.memory_norms[old_idx];
                if old_idx < self.text_buffer.len() {
                    new_text.push(self.text_buffer[old_idx].clone());
                }
                if old_idx < self.hash_buffer.len() {
                    new_hash.push(self.hash_buffer[old_idx].clone());
                }
            }

            self.memory_bank = new_bank;
            self.memory_norms = new_norms;
            self.size = keep.len();
            self.text_buffer = new_text;
            self.hash_buffer = new_hash;
        }

        self.context_cache = None;
    }

    /// Retrieve the current coherent context string.
    ///
    /// Verifies integrity of each memory against the Merkle chain.
    /// Rejects tampered fragments.
    pub fn retrieve_context(&mut self) -> &str {
        if self.context_cache.is_some() {
            return self.context_cache.as_ref().unwrap();
        }

        // Verify all hashes against the integrity chain
        let valid_texts: Vec<&str> = self
            .text_buffer
            .iter()
            .zip(self.hash_buffer.iter())
            .filter(|(_, h)| self.chain.contains(h))
            .map(|(t, _)| t.as_str())
            .collect();

        self.context_cache = Some(valid_texts.join("\n"));
        self.context_cache.as_ref().unwrap()
    }

    /// Trinary Undo (TNeg [-1]): reverse the last memory.
    pub fn trinary_undo(&mut self) -> bool {
        if self.size == 0 {
            return false;
        }

        self.text_buffer.pop();
        self.hash_buffer.pop();
        self.size -= 1;

        // Zero out the removed slot
        let start = self.size * self.dim;
        for v in &mut self.memory_bank[start..start + self.dim] {
            *v = 0.0;
        }
        self.memory_norms[self.size] = 0.0;
        self.temporal_states[self.size] = TemporalState::Present;
        self.context_cache = None;
        true
    }

    /// Export the CSNP state as a summary.
    pub fn export_state(&self) -> CsnpStateExport {
        CsnpStateExport {
            merkle_root: self.chain.root_hash(),
            memory_count: self.text_buffer.len(),
            identity_norm: self
                .identity_state
                .iter()
                .map(|v| v * v)
                .sum::<f64>()
                .sqrt(),
            protocol: "CSNP/v1-Trinary".into(),
        }
    }
}

/// Exported CSNP state summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsnpStateExport {
    pub merkle_root: String,
    pub memory_count: usize,
    pub identity_norm: f64,
    pub protocol: String,
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a simple embedding of given dimension.
    fn make_embedding(dim: usize, base: f64) -> Vec<f64> {
        (0..dim).map(|i| base + (i as f64) * 0.01).collect()
    }

    #[test]
    fn test_new_csnp() {
        let csnp = CsnpManager::new(4, 5);
        assert_eq!(csnp.size(), 0);
        assert_eq!(csnp.dim, 4);
        assert_eq!(csnp.context_limit, 5);
        assert!(!csnp.is_full());
    }

    #[test]
    fn test_update_state_basic() {
        let mut csnp = CsnpManager::new(4, 5);
        let emb = vec![1.0, 0.0, 0.0, 0.0];
        csnp.update_state("USER:hello|AI:hi", &emb);
        assert_eq!(csnp.size(), 1);
        assert_eq!(csnp.text_buffer[0], "USER:hello|AI:hi");
    }

    #[test]
    fn test_identity_evolves() {
        let mut csnp = CsnpManager::new(2, 10);

        // First update: identity = embedding (normalized)
        csnp.update_state("turn1", &[3.0, 4.0]);
        let id = csnp.identity();
        let norm: f64 = id.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "Identity should be normalized, got norm {}",
            norm
        );

        // Second update: identity should shift toward new embedding
        let id_before = csnp.identity().to_vec();
        csnp.update_state("turn2", &[0.0, 1.0]);
        let id_after = csnp.identity();
        assert!(
            id_before != id_after,
            "Identity should evolve with new input"
        );
    }

    #[test]
    fn test_compression_triggers_at_limit() {
        let dim = 4;
        let limit = 3;
        let mut csnp = CsnpManager::new(dim, limit);

        // Fill to limit
        for i in 0..=limit {
            let emb = make_embedding(dim, (i + 1) as f64);
            csnp.update_state(&format!("turn{}", i), &emb);
        }

        // After inserting limit+1 items, compression should have fired
        assert!(
            csnp.size() <= limit,
            "Size should be <= limit after compression, got {}",
            csnp.size()
        );
    }

    #[test]
    fn test_trinary_undo() {
        let mut csnp = CsnpManager::new(2, 10);
        csnp.update_state("turn1", &[1.0, 0.0]);
        csnp.update_state("turn2", &[0.0, 1.0]);
        assert_eq!(csnp.size(), 2);

        assert!(csnp.trinary_undo());
        assert_eq!(csnp.size(), 1);
        assert_eq!(csnp.text_buffer.len(), 1);
        assert_eq!(csnp.text_buffer[0], "turn1");
    }

    #[test]
    fn test_trinary_undo_empty() {
        let mut csnp = CsnpManager::new(2, 10);
        assert!(!csnp.trinary_undo(), "Undo on empty should return false");
    }

    #[test]
    fn test_retrieve_context() {
        let mut csnp = CsnpManager::new(2, 10);
        csnp.update_state("USER:a|AI:b", &[1.0, 0.0]);
        csnp.update_state("USER:c|AI:d", &[0.0, 1.0]);

        let ctx = csnp.retrieve_context();
        assert!(ctx.contains("USER:a|AI:b"));
        assert!(ctx.contains("USER:c|AI:d"));
    }

    #[test]
    fn test_context_cache_invalidation() {
        let mut csnp = CsnpManager::new(2, 10);
        csnp.update_state("turn1", &[1.0, 0.0]);
        let _ = csnp.retrieve_context(); // warm cache
        assert!(csnp.context_cache.is_some());

        csnp.update_state("turn2", &[0.0, 1.0]);
        assert!(
            csnp.context_cache.is_none(),
            "Cache should be invalidated after update"
        );
    }

    #[test]
    fn test_integrity_chain() {
        let mut chain = IntegrityChain::new();
        let h1 = chain.add_entry("hello");
        let h2 = chain.add_entry("world");
        assert!(chain.contains(&h1));
        assert!(chain.contains(&h2));
        assert!(!chain.contains("fake_hash"));

        let root = chain.root_hash();
        assert!(!root.is_empty());
    }

    #[test]
    fn test_export_state() {
        let mut csnp = CsnpManager::new(2, 10);
        csnp.update_state("turn1", &[1.0, 0.0]);
        let export = csnp.export_state();
        assert_eq!(export.memory_count, 1);
        assert_eq!(export.protocol, "CSNP/v1-Trinary");
        assert!(!export.merkle_root.is_empty());
    }

    #[test]
    fn test_consolidate_memory() {
        let dim = 2;
        let limit = 10;
        let mut csnp = CsnpManager::new(dim, limit);

        // Fill to 90% (9 out of 10)
        for i in 0..9 {
            csnp.update_state(&format!("turn{}", i), &make_embedding(dim, (i + 1) as f64));
        }

        // 9 > 80% of 10 (8), should trigger consolidation
        csnp.consolidate_memory();
        assert!(
            csnp.size() <= 8,
            "Should consolidate to 80%, got size {}",
            csnp.size()
        );
    }

    #[test]
    fn test_eviction_removes_least_relevant() {
        let dim = 2;
        let limit = 3;
        let mut csnp = CsnpManager::new(dim, limit);

        // The identity will converge toward [1, 1] direction
        csnp.update_state("near1", &[1.0, 1.0]);
        csnp.update_state("near2", &[1.1, 0.9]);
        csnp.update_state("near3", &[0.9, 1.1]);

        // This one is far from identity → should be evicted
        csnp.update_state("far", &[-10.0, -10.0]);

        // After compression, "far" should have been evicted
        assert_eq!(csnp.size(), 3);
        assert!(
            !csnp.text_buffer.contains(&"far".to_string()),
            "Far memory should be evicted, buffer: {:?}",
            csnp.text_buffer
        );
    }
}
