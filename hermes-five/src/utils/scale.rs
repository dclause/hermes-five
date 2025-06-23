use num_traits::ToPrimitive;

/// Trait to convert from f64 to a numeric type with saturation.
///
/// # Notes
/// - For integer types, the value is rounded to the nearest integer.
/// - No explicit handling of NaN or infinite values: results are undefined for those inputs.
/// - Values outside the target type range saturate to min/max of the type.
/// - Behavior on NaN or infinite inputs is platform-dependent and should be avoided unless documented.
///
/// This trait prioritizes speed and assumes inputs are "well-formed" finite numbers.
pub trait FromF64Sat: Sized {
    /// Convert from `f64` to `Self` with saturation.
    fn from_f64_sat(value: f64) -> Self;
}

macro_rules! impl_from_f64_sat_int {
    ($($t:ty),*) => {
        $(
            impl FromF64Sat for $t {
                #[inline(always)]
                fn from_f64_sat(value: f64) -> Self {
                    // Round value to nearest integer
                    let val = value.round();
                    if val < <$t>::MIN as f64 {
                        <$t>::MIN
                    } else if val > <$t>::MAX as f64 {
                        <$t>::MAX
                    } else {
                        val as $t
                    }
                }
            }
        )*
    };
}

macro_rules! impl_from_f64_sat_float {
    ($($t:ty),*) => {
        $(
            impl FromF64Sat for $t {
                #[inline(always)]
                fn from_f64_sat(value: f64) -> Self {
                    // Direct cast, no saturation needed for floats.
                    // Note: NaN and infinite values preserved as-is.
                    value as $t
                }
            }
        )*
    };
}

impl_from_f64_sat_int!(u8, u16, u32, u64, i8, i16, i32, i64);
impl_from_f64_sat_float!(f32, f64);

/// Trait for mapping a numeric value linearly from one scale to another,
/// similar to Arduino's map() function.
///
/// # Behavior and guarantees:
/// - Linear mapping from `[from_low, from_high]` to `[to_low, to_high]`.
/// - Supports inverted source or target ranges (from_high < from_low or to_high < to_low).
/// - If `from_low == from_high`, returns `to_low` directly to avoid division by zero.
/// - Values outside the source range are extrapolated (no clipping).
/// - Undefined behavior if any input is NaN or infinite (caller responsibility).
/// - Conversion errors are considered impossible for standard numeric types used.
///
/// # Performance notes:
/// - Uses f64 internally for calculation to balance precision and performance.
/// - Conversion to f64 assumed infallible for numeric primitives.
/// - Minimal branching for speed.
///
/// # Type parameters:
/// - `R`: Target numeric type implementing `ToPrimitive` and `FromF64Sat`.
pub trait Scalable: ToPrimitive + Sized {
    fn scale<R: ToPrimitive + FromF64Sat>(
        self,
        from_low: Self,
        from_high: Self,
        to_low: R,
        to_high: R,
    ) -> R;
}

macro_rules! impl_scalable {
    ($($t:ty),*) => {
        $(
            impl Scalable for $t {
                #[inline(always)]
                fn scale<R: ToPrimitive + FromF64Sat>(
                    self,
                    from_low: $t,
                    from_high: $t,
                    to_low: R,
                    to_high: R,
                ) -> R {
                    // Convert inputs to f64 once for calculation
                    let self_f = self.to_f64().unwrap();
                    let from_low_f = from_low.to_f64().unwrap();
                    let from_high_f = from_high.to_f64().unwrap();
                    let to_low_f = to_low.to_f64().unwrap();
                    let to_high_f = to_high.to_f64().unwrap();

                    let denom = from_high_f - from_low_f;

                    // If denom ~ 0, return to_low_f directly
                    let result = if denom.abs() < f64::EPSILON {
                        to_low_f
                    } else {
                        (self_f - from_low_f) * (to_high_f - to_low_f) / denom + to_low_f
                    };

                    R::from_f64_sat(result)
                }
            }
        )*
    };
}

impl_scalable!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[cfg(test)]
mod tests {
    use super::Scalable;
    use std::f64;

    // Helper: test float with epsilon equality.
    fn approx_eq(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn test_scale_unsigned_normal() {
        assert_eq!(0.scale::<u8>(0, 100, 0, 255), 0);
        assert_eq!(50.scale::<u8>(0, 100, 0, 255), 128);
        assert_eq!(100.scale::<u16>(0, 100, 0, 5000), 5000);
    }

    #[test]
    fn test_scale_signed_normal() {
        assert_eq!(0.scale::<i8>(0, 100, -50, 0), -50);
        assert_eq!(50.scale::<i8>(0, 100, -50, 50), 0);
        assert_eq!(100.scale::<i8>(0, 100, -50, 50), 50);
    }

    #[test]
    fn test_scale_float_normal() {
        assert!(approx_eq(
            0.5.scale::<f32>(0.0, 1.0, 0.0, 100.0) as f64,
            50.0,
            f32::EPSILON as f64
        ));
        assert!(approx_eq(
            1.0.scale::<f64>(0.0, 1.0, 0.0, 100.0),
            100.0,
            f64::EPSILON
        ));
    }

    #[test]
    fn test_scale_inverse_range_source() {
        // from_high < from_low
        assert_eq!(0.scale::<u64>(100, 0, 180, 0), 0);
        assert_eq!(100.scale::<u64>(100, 0, 180, 0), 180);
    }

    #[test]
    fn test_scale_inverse_range_target() {
        // to_high < to_low
        assert_eq!(0.scale::<u16>(0, 100, 180, 0), 180);
        assert_eq!(100.scale::<u16>(0, 100, 180, 0), 0);
    }

    #[test]
    fn test_scale_division_by_zero_source_range() {
        // from_low == from_high
        // Should return to_low by design
        assert_eq!(50.scale::<u8>(10, 10, 0, 100), 0);
        assert_eq!(0.scale::<i32>(5, 5, -100, 100), -100);
        assert_eq!(5.scale::<f64>(5, 5, 100.0, 100.0), 100.0);
    }

    #[test]
    fn test_scale_outside_source_range_clipping() {
        // Values lower than from_low -> clipped to to_low (via formula)
        assert_eq!(u8::MIN.scale::<u8>(10, 100, 0, 200), 0);
        assert_eq!(5.scale::<u8>(10, 100, 0, 200), 0);

        // Values higher than from_high -> clipped to to_high (via formula)
        assert_eq!(150.scale::<u8>(0, 100, 0, 200), 255);
        assert_eq!(u8::MAX.scale::<u8>(0, 100, 0, 200), 255);
    }

    #[test]
    fn test_scale_clipping_min_max_types() {
        // Check clipping to min/max on integer types explicitly
        let below_min = (i8::MIN as f64) - 10.0;
        let above_max = (i8::MAX as f64) + 10.0;

        assert_eq!(below_min.scale::<i8>(0.0, 1.0, i8::MIN, i8::MAX), i8::MIN);
        assert_eq!(above_max.scale::<i8>(0.0, 1.0, i8::MIN, i8::MAX), i8::MAX);
    }

    #[test]
    fn test_scale_nan_and_infinity() {
        use std::f64::{INFINITY, NAN, NEG_INFINITY};

        // NaN propagates, but from_f64_sat should convert NaN to 0 or min?
        // Let's test the behavior (likely NaN -> 0 or panic)
        // Since from_f64_sat just does value as t for floats, NaN as f32/f64 remains NaN
        // For integers, NaN.round() is also NaN -> likely 0 as integer
        // So we test floats explicitly here

        assert!(NAN.scale::<f32>(0.0, 1.0, 0.0, 100.0).is_nan());
        assert!(INFINITY.scale::<f64>(0.0, 1.0, 0.0, 100.0).is_infinite());
        assert!(NEG_INFINITY
            .scale::<f64>(0.0, 1.0, 0.0, 100.0)
            .is_infinite());
    }

    #[test]
    fn test_scale_rounding_behavior() {
        // Values that should round up or down
        assert_eq!(0.4999.scale::<u8>(0.0, 1.0, 0, 255), 127);
        assert_eq!(0.5.scale::<u8>(0.0, 1.0, 0, 255), 128);
        assert_eq!(0.5001.scale::<u8>(0.0, 1.0, 0, 255), 128);
    }

    #[test]
    fn test_scale_f64_to_integer() {
        // Test conversion from f64 to integer via scale
        assert_eq!(0.75.scale::<u16>(0.0, 1.0, 0, 180), 135);
        assert_eq!(0.75.scale::<i16>(0.0, 1.0, 180, 0), 45);
    }

    #[test]
    fn test_scale_signed_negative_ranges() {
        assert_eq!((-0.75).scale::<i16>(0.0, -1.0, 0, 180), 135);
        assert_eq!((-0.25).scale::<i16>(-1.0, 0.0, 180, 0), 45);
    }

    #[test]
    fn test_scale_floats_large_range() {
        // test large float range mapping
        let result = 1_000_000.0.scale::<f64>(0.0, 1_000_000.0, 0.0, 10_000.0);
        assert!(approx_eq(result, 10_000.0, 1e-9));
    }
}
