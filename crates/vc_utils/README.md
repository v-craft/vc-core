# Platform-agnostic Utilities

*Platform-agnostic: No dependencies on atomic variables, sync primitives, or OS-APIs like threads and timing.*

Includes the following components:

- **Hash Containers**:
    - Re-exports of `hashbrown` and `foldhash`
    - Newtype wrappers based on fixed hash states
    - Pre-hashed containers and no-op hash calculators
- **Type ID Tables**:
    - Maps using `TypeId` as keys
- **Stack-optimized Linear Collections**:
    - Re-exports of `fastvec`
- **`range_invoke`**:
    - Helper macros
- **`default`**:
    - Simplified `Default::default()` usage

Platform-specific additional containers are available in `vc_os::utils`.
