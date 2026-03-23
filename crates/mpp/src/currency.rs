//! USD / USDC conversion utilities.
//!
//! USDC uses 6 decimal places: 1_000_000 base units = $1.00.

use alloy::primitives::U256;

/// Convert USD (f64) to USDC base units (U256, 6 decimals).
///
/// ```
/// use mpp::currency::usdc_from_usd;
/// use alloy::primitives::U256;
///
/// let amount = usdc_from_usd(1.50);
/// assert_eq!(amount, U256::from(1_500_000u64));
/// ```
pub fn usdc_from_usd(usd: f64) -> U256 {
    U256::from((usd * 1_000_000.0) as u64)
}

/// Convert USDC base units (U256) back to USD (f64).
///
/// ```
/// use mpp::currency::usd_from_usdc;
/// use alloy::primitives::U256;
///
/// let usd = usd_from_usdc(U256::from(2_500_000u64));
/// assert!((usd - 2.50).abs() < 0.000001);
/// ```
pub fn usd_from_usdc(usdc: U256) -> f64 {
    let micro: u64 = usdc.try_into().unwrap_or(u64::MAX);
    micro as f64 / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let usd = 1.234567;
        let usdc = usdc_from_usd(usd);
        let back = usd_from_usdc(usdc);
        assert!((back - usd).abs() < 0.000001);
    }

    #[test]
    fn zero() {
        assert_eq!(usdc_from_usd(0.0), U256::ZERO);
        assert!((usd_from_usdc(U256::ZERO)).abs() < f64::EPSILON);
    }

    #[test]
    fn one_dollar() {
        assert_eq!(usdc_from_usd(1.0), U256::from(1_000_000u64));
    }
}
