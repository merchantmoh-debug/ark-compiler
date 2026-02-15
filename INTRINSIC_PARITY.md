# Intrinsic Parity Ledger: Python (`meta/ark.py`) vs Rust (`core/src/intrinsics.rs`)

**Generated:** 2026-02-15 | **Phase:** 72 (Structural Hardening)

> Track every intrinsic. Close the gap. No orphans.

## Legend

| Status | Meaning |
|---|---|
| ✅ | **PARITY** — Exists in both Python and Rust |
| ⚠️ | **PYTHON_ONLY** — Exists only in `ark.py` (debt) |
| 🔲 | **STUB** — Rust function exists but returns `unimplemented!()` |

---

## Core Operators

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `intrinsic_add` | ✅ | ✅ | ✅ |
| `intrinsic_sub` | ✅ | ✅ | ✅ |
| `intrinsic_mul` | ✅ | ✅ | ✅ |
| `intrinsic_div` | ✅ | ✅ | ✅ |
| `intrinsic_mod` | ✅ | ✅ | ✅ |
| `intrinsic_gt` | ✅ | ✅ | ✅ |
| `intrinsic_lt` | ✅ | ✅ | ✅ |
| `intrinsic_ge` | ✅ | ✅ | ✅ |
| `intrinsic_le` | ✅ | ✅ | ✅ |
| `intrinsic_eq` | ✅ | ✅ | ✅ |
| `intrinsic_and` | ✅ | ✅ | ✅ |
| `intrinsic_or` | ✅ | ✅ | ✅ |
| `intrinsic_not` | ✅ | ✅ | ✅ |
| `print` | ✅ | ✅ | ✅ |

## I/O & File System

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.fs.read` | ✅ | ✅ | ✅ |
| `sys.fs.write` | ✅ | ✅ | ✅ |
| `sys.fs.read_buffer` | ✅ | ✅ | ✅ |
| `sys.fs.write_buffer` | ✅ | ✅ | ✅ |
| `sys.io.read_bytes` | ✅ | ❌ | ⚠️ |
| `sys.io.read_line` | ✅ | ❌ | ⚠️ |
| `sys.io.write` | ✅ | ❌ | ⚠️ |
| `sys.io.read_file_async` | ✅ | ❌ | ⚠️ |
| `sys.exec` | ✅ | ✅ | ✅ |

## Cryptography

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.crypto.hash` | ✅ | ✅ | ✅ |
| `sys.crypto.verify` | ✅ (via `ed25519.verify`) | ✅ | ✅ |
| `sys.crypto.merkle_root` | ✅ | ✅ | ✅ |
| `sys.crypto.sha512` | ✅ | ❌ | ⚠️ |
| `sys.crypto.hmac_sha512` | ✅ | ❌ | ⚠️ |
| `sys.crypto.pbkdf2_hmac_sha512` | ✅ | ❌ | ⚠️ |
| `sys.crypto.aes_gcm_encrypt` | ✅ | ❌ | ⚠️ |
| `sys.crypto.aes_gcm_decrypt` | ✅ | ❌ | ⚠️ |
| `sys.crypto.random_bytes` | ✅ | ❌ | ⚠️ |
| `sys.crypto.ed25519.gen` | ✅ | ❌ | ⚠️ |
| `sys.crypto.ed25519.sign` | ✅ | ❌ | ⚠️ |
| `sys.crypto.ed25519.verify` | ✅ | ❌ | ⚠️ |

## Math

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `math.pow` | ✅ | ✅ | ✅ |
| `math.sqrt` | ✅ | ✅ | ✅ |
| `math.sin` | ✅ | ✅ | ✅ |
| `math.cos` | ✅ | ✅ | ✅ |
| `math.tan` | ✅ | ✅ | ✅ |
| `math.asin` | ✅ | ✅ | ✅ |
| `math.acos` | ✅ | ✅ | ✅ |
| `math.atan` | ✅ | ✅ | ✅ |
| `math.atan2` | ✅ | ✅ | ✅ |
| `math.sin_scaled` | ✅ | ✅ | ✅ |
| `math.cos_scaled` | ✅ | ✅ | ✅ |
| `math.pi_scaled` | ✅ | ✅ | ✅ |
| `sys.math.pow_mod` | ✅ | ❌ | ⚠️ |

## Memory & Buffers

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.mem.alloc` | ✅ | ✅ | ✅ |
| `sys.mem.inspect` | ✅ | ✅ | ✅ |
| `sys.mem.read` | ✅ | ✅ | ✅ |
| `sys.mem.write` | ✅ | ✅ | ✅ |

## Lists & Structs

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.list.get` | ✅ | ✅ | ✅ |
| `sys.list.set` | ✅ | ✅ | ✅ |
| `sys.list.append` | ✅ | ✅ | ✅ |
| `sys.len` | ✅ | ✅ | ✅ |
| `sys.list.pop` | ✅ | ❌ | ⚠️ |
| `sys.list.delete` | ✅ | ❌ | ⚠️ |
| `sys.struct.get` | ✅ | ✅ | ✅ |
| `sys.struct.set` | ✅ | ✅ | ✅ |
| `sys.struct.has` | ✅ | ❌ | ⚠️ |
| `sys.str.get` | ✅ | ✅ | ✅ |
| `sys.str.from_code` | ✅ | ✅ | ✅ |

## Networking

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.net.http.request` | ✅ | ❌ | ⚠️ |
| `sys.net.http.serve` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.bind` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.accept` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.connect` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.send` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.recv` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.close` | ✅ | ❌ | ⚠️ |
| `sys.net.socket.set_timeout` | ✅ | ❌ | ⚠️ |

## Blockchain / Chain

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.chain.height` | ✅ | ✅ | ✅ |
| `sys.chain.get_balance` | ✅ | ✅ | ✅ |
| `sys.chain.submit_tx` | ✅ | ✅ | ✅ |
| `sys.chain.verify_tx` | ✅ | ✅ | ✅ |

## System & Runtime

| Intrinsic | Python | Rust | Status |
|---|---|---|---|
| `sys.time.now` | ✅ | ✅ | ✅ |
| `sys.time.sleep` | ✅ | ❌ | ⚠️ |
| `sys.json.parse` | ✅ | ❌ | ⚠️ |
| `sys.json.stringify` | ✅ | ❌ | ⚠️ |
| `sys.log` | ✅ | ❌ | ⚠️ |
| `sys.exit` | ✅ | ❌ | ⚠️ |
| `sys.vm.eval` | ✅ | ❌ | ⚠️ |
| `sys.vm.source` | ✅ | ❌ | ⚠️ |
| `sys.event.poll` | ✅ | ❌ | ⚠️ |
| `sys.func.apply` | ✅ | ❌ | ⚠️ |
| `sys.thread.spawn` | ✅ | ❌ | ⚠️ |
| `intrinsic_ask_ai` | ✅ | ✅ | ✅ |
| `io.cls` | ✅ | ✅ | ✅ |
| `intrinsic_extract_code` | ✅ | ❌ | ⚠️ |

---

## Summary

| Status | Count |
|---|---|
| ✅ PARITY | **55** |
| ⚠️ PYTHON_ONLY | **40** |
| 🔲 STUB | **0** |
| **Total** | **95** |

**Parity Ratio: 57.9%** — Target: 80%+ by Phase 75.

### Priority Debt (Must port to Rust for WASM viability)

1. `sys.json.parse` / `sys.json.stringify` — Required for all WASM FFI
2. `sys.list.pop` / `sys.list.delete` — Common list ops
3. `sys.exit` — Basic program control
4. `sys.time.sleep` — Used in async/network tests
5. `sys.log` — Debugging
