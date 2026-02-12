# Swarm Plan: The Sovereign Standard Library (@[/ARK_PRIME])

**Objective:** Create `lib/std` modules to wrap low-level intrinsics.
**Constraint:** Must be written in Ark.
**Protocol:** ARK OMEGA-POINT (Tier-0).

## 1. The Modules (The Hands)

| Module | File | Wraps | Features |
| :--- | :--- | :--- | :--- |
| **IO** | `lib/std/io.ark` | `intrinsic_print`, `io.cls` | `print`, `println`, `clear`. |
| **FS** | `lib/std/fs.ark` | `sys.fs.read`, `sys.fs.write` | `read_file`, `write_file`, `append_file`. |
| **Net** | `lib/std/net.ark` | `sys.net.*` (If avail) | `http.get`, `http.post`. |
| **Math** | `lib/std/math.ark` | `math.pow`, `math.sqrt` | `pow`, `sqrt`, `abs`. |
| **AI** | `lib/std/ai.ark` | `intrinsic_ask_ai` | `ask(prompt)`, `Agent` struct. |
| **Core** | `lib/std/core.ark` | `sys.len` | `len`, `range`, `assert`. |

## 2. The Compiler Bridge (The Nervous System)

*   **`meta/compile.py`**: Must be updated to resolve `import std.io` to `lib/std/io.ark`.
*   **Current State:** Checking `compile.py` for `include` or `import` logic.
*   **Action:** Patch `compile.py` to support library paths.

## 3. Execution (The Swarm)

1.  **Create Directory:** `mkdir lib/std`
2.  **Inject Code:** Write all `.ark` files simultaneously.
3.  **Patch Compiler:** Update Python bootstrap to link them.
4.  **Verify:** Run `apps/test_std.ark`.
