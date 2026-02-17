#!/bin/bash
# ══════════════════════════════════════════════════════════════
# P2: Git Cleanup — Remove committed debris from tracking
# ══════════════════════════════════════════════════════════════
# This script removes files that should never have been committed.
# It does NOT delete them from disk — only from git tracking.
# After running, commit with: git commit -m "chore: remove tracked debris"
# ══════════════════════════════════════════════════════════════
set -e

echo "Removing tracked debris files..."

# Test artifacts
git rm --cached --ignore-unmatch \
    test.mp3 \
    test_tone.wav \
    bench_temp.txt \
    test_data.txt \
    test_io.tmp \
    test_memory.enc \
    2>/dev/null || true

# Debug artifacts
git rm --cached --ignore-unmatch \
    debug_build.py \
    debug_output.txt \
    2>/dev/null || true

# Build/operational outputs
git rm --cached --ignore-unmatch \
    out.json \
    nts_clean.txt \
    security.json \
    build_log.txt \
    trace.log \
    2>/dev/null || true

# Swarm logs
git rm --cached --ignore-unmatch \
    swarm_dispatch_log.json \
    swarm_wave2_log.json \
    swarm_wave3_log.json \
    swarm_wave4_log.json \
    2>/dev/null || true

# Integration logs
git rm --cached --ignore-unmatch \
    test_integration.log \
    test_integration_2.log \
    test_integration_3.log \
    test_integration_4.log \
    test_integration_5.log \
    test_integration_6.log \
    test_integration_7.log \
    2>/dev/null || true

echo ""
echo "Done. Now run:"
echo "  git add .gitignore"
echo "  git commit -m 'chore: remove tracked debris, update .gitignore'"
