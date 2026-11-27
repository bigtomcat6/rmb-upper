// Shared fixtures for the integration tests. Each test binary that includes
// this module uses only part of it, so unused helpers are expected.
#![allow(dead_code)]

use rmb_upper::MAX_CENTS;

/// Every cent value from 0 through 200_000, followed by ten thousand
/// deterministic pseudo-random values in `0..=MAX_CENTS`.
pub fn sample_cents() -> Vec<i128> {
    let mut values: Vec<i128> = (0..=200_000).collect();
    let mut state = 0x1234_5678_9abc_def0_u64;
    let range = MAX_CENTS as u64 + 1;

    for _ in 0..10_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        values.push((state % range) as i128);
    }

    values
}

/// Formats a cent value the way `format_decimal` expects it, e.g. `12345`
/// becomes `"123.45"`.
pub fn decimal_string(cents: i128) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}
