# A. Numeric Casting Rules

The language differentiates between Preserving (Safe) casts and Reinterpreting (Unsafe/Lossy) casts.

| **Source** | **Target** | **Safety**       | **Logic**                                                                                 |
| ---------- | ---------- | ---------------- | ----------------------------------------------------------------------------------------- |
| `u8`       | `u16`      | **Safe**         | Zero-extension. No data loss.                                                             |
| `i16`      | `i32`      | **Safe**         | Sign-extension. No data loss.                                                             |
| `u32`      | `u16`      | **Lossy**        | High bits are truncated.                                                                  |
| `i32`      | `u32`      | **SignMismatch** | Bitwise identical, but `-1` becomes `4294967295`.                                         |
| `f64`      | `i64`      | **FloatToInt**   | Decimal part discarded. Saturates or wraps on overflow.                                   |
| `i64`      | `f64`      | **IntToFloat**   | Valid, but logic checks required (53-bit mantissa cannot hold full 64-bit int precision). |

# B. Assignment Rules (Implicit)

To prevent bugs, implicit casting (assignment without `as`) is strictly limited:

1. **Identity**: `T a = b` is valid if `typeof(a) == typeof(b)`.
2. **Void Pointers**: `void*` can implicitly accept any pointer type.
    - *Rationale*: This allows generic memory functions (`malloc`, `memcpy`) to work without cluttering code with casts.
3. **Strictness**: `i32` cannot be assigned to `u32` implicitly. `u64` cannot be assigned to `usize` implicitly (architecture dependency). // `usize` is not implemented for now

# C. Pointer Rules
Low-level control is required for memory-mapped I/O and allocators.
1. **Pointer -> Integer**: Explicit cast (`as`) required.
2. **Integer -> Pointer**: Explicit cast (`as`) required.
3. **Pointer -> Pointer**: Explicit cast (`as`) required.
   - *Exception*: `*void`.
