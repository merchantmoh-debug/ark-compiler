# buffer_fix.md (Micro-Plan)

## Goal
Fix `intrinsic_buffer_write` to allow in-place mutation by returning the buffer.

## Changes
### `core/src/intrinsics.rs`
- Modify `intrinsic_buffer_write(args)`:
    - Expects `[buffer, index, value]`.
    - Extract buffer (linear).
    - Mutate.
    - Return `buffer`.

## Usage
Old: `sys.mem.write(buf, 0, 10)` (Ineffective if shared/cloned)
New: `buf := sys.mem.write(buf, 0, 10)` (Linear Swap)

## Verification
- Create `apps/test_buffer.ark`.
