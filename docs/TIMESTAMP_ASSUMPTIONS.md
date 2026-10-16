# Timestamp Assumptions and Boundaries

This document describes timestamp-related assumptions and boundary conditions for RetinaX contracts.

## Core Assumptions

### 1. Monotonicity
**Assumption**: Block timestamps are monotonically increasing.
- Each block's timestamp >= previous block's timestamp
- Rollbacks (timestamp going backwards) are not expected
- **Risk**: If timestamp rolls back, time-dependent logic may fail

### 2. Timestamp Range
**Range**: 0 to u64::MAX (0 to 18,446,744,073,709,551,615)
- **Start**: Contracts may see timestamp 0 in test environments
- **End**: u64::MAX represents ~584 billion years from Unix epoch
- **Practical**: Current timestamps are ~1.7 billion (year 2024)

### 3. Timestamp Arithmetic
**Operations**: All timestamp arithmetic uses checked operations
- `checked_add()` for future timestamps
- `checked_sub()` for duration calculations
- `saturating_sub()` for duration when order uncertain

## Contracts Using Timestamps

### Timelock (`contracts/timelock/timelock.rs`)
- **Usage**: `execute_after` comparison
- **Boundary**: Must handle timestamp 0 and u64::MAX
- **Test**: `timestamp_tests.rs`

### Staking (`contracts/staking/src/lib.rs`)
- **Usage**: Loyalty multiplier time calculations
- **Calculation**: `(current_time - stake_start) / seconds_per_day`
- **Boundary**: Must not overflow on long staking periods
- **Test**: `timestamp_tests.rs`

### Consent (`contracts/common/src/consent.rs`)
- **Usage**: `expires_at` comparison
- **Boundary**: `current_time >= expires_at`
- **Test**: Covered in consent tests

### Session (`contracts/common/src/session.rs`)
- **Usage**: Session expiry validation
- **Boundary**: `current_time < session.expires_at`
- **Test**: Covered in session tests

### Rate Limit (`contracts/common/src/rate_limit.rs`)
- **Usage**: Rate window calculations
- **Calculation**: `current_time - window_start`
- **Boundary**: Must handle rapid requests at boundaries
- **Test**: Covered in rate limit tests

## Edge Cases Tested

### 1. Timestamp = 0
```rust
let current_time = 0u64;
let execute_after = 1000u64;
assert!(current_time < execute_after);
```

### 2. Timestamp = u64::MAX
```rust
let current_time = u64::MAX;
let execute_after = 1000u64;
assert!(current_time >= execute_after);
```

### 3. Just Before Expiry
```rust
let current_time = 999u64;
let execute_after = 1000u64;
assert!(current_time < execute_after);
```

### 4. Exactly At Expiry
```rust
let current_time = 1000u64;
let execute_after = 1000u64;
assert!(current_time >= execute_after);
```

### 5. Just After Expiry
```rust
let current_time = 1001u64;
let execute_after = 1000u64;
assert!(current_time >= execute_after);
```

## Overflow Protection

All duration and future timestamp calculations use checked arithmetic:

```rust
// Good: Checked addition
let future_time = current_time.checked_add(duration)?;

// Good: Checked subtraction
let elapsed = current_time.checked_sub(start_time)?;

// Good: Saturating subtraction (when negative not expected)
let elapsed = current_time.saturating_sub(start_time);
```

## Testing Strategy

1. **Boundary Tests**: Test at 0, u64::MAX, and expiry boundaries
2. **Overflow Tests**: Verify checked arithmetic catches overflows
3. **Monotonicity Tests**: Verify timestamp ordering assumptions
4. **Edge Case Tests**: Test before/at/after expiry conditions

## Remediation for Timestamp Issues

If timestamp issues are discovered:

1. **Never panic** on timestamp edge cases
2. **Use checked arithmetic** for all calculations
3. **Document assumptions** about timestamp behavior
4. **Test all boundaries** in unit tests
5. **Handle gracefully** if assumptions violated
