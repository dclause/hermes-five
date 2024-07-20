/// Trait for mapping a value from one scale to another.
pub trait Scalable {
    /// Map a value from one scale to another.
    /// This is equivalent to Arduino map() method:
    /// https://www.arduino.cc/reference/en/language/functions/math/map/
    ///
    /// # Parameters
    /// * `self`:  the value to map
    /// * `from_low`:  the low end of the originating range
    /// * `from_high`:  the high end of the originating range
    /// * `to_low`:  the low end of the target range
    /// * `to_high`:  the high end of the target range
    ///
    /// # Returns
    /// The mapped value.
    fn scale(self, from_low: Self, from_high: Self, to_low: Self, to_high: Self) -> Self;
}

macro_rules! impl_from_scalable {
    ($($variant:ty),*) => {
        $(
            impl Scalable for $variant {
                fn scale(self, from_low: Self, from_high: Self, to_low: Self, to_high: Self) -> Self {
                    ((self as f64 - from_low as f64) * (to_high as f64 - to_low as f64)
                        / (from_high as f64 - from_low as f64)
                        + to_low as f64) as Self
                }
            }
        )*
    };
}

// Implement trait for all number types.
impl_from_scalable!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[cfg(test)]
mod tests {
    use super::Scalable;

    #[test]
    fn test_scale_u8() {
        assert_eq!(50u8.scale(0, 100, 0, 255), 127);
        assert_eq!(0u8.scale(0, 100, 0, 255), 0);
        assert_eq!(100u8.scale(0, 100, 0, 255), 255);
    }

    #[test]
    fn test_scale_i32() {
        assert_eq!(50i32.scale(0, 100, -100, 100), 0);
        assert_eq!(0i32.scale(0, 100, -100, 100), -100);
        assert_eq!(100i32.scale(0, 100, -100, 100), 100);
    }

    #[test]
    fn test_scale_f32() {
        assert!((0.5f32.scale(0.0, 1.0, 0.0, 100.0) - 50.0).abs() < f32::EPSILON);
        assert!((0.0f32.scale(0.0, 1.0, 0.0, 100.0) - 0.0).abs() < f32::EPSILON);
        assert!((1.0f32.scale(0.0, 1.0, 0.0, 100.0) - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scale_f64() {
        assert!((0.5f64.scale(0.0, 1.0, 0.0, 100.0) - 50.0).abs() < f64::EPSILON);
        assert!((0.0f64.scale(0.0, 1.0, 0.0, 100.0) - 0.0).abs() < f64::EPSILON);
        assert!((1.0f64.scale(0.0, 1.0, 0.0, 100.0) - 100.0).abs() < f64::EPSILON);
    }
}
