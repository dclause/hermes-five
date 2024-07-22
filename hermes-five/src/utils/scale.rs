pub trait ToF64 {
    fn to_f64(self) -> f64;
}

pub trait FromF64 {
    fn from_f64(value: f64) -> Self;
}

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
    fn scale<R: FromF64>(
        self,
        from_low: impl ToF64,
        from_high: impl ToF64,
        to_low: impl ToF64,
        to_high: impl ToF64,
    ) -> R;
}

macro_rules! impl_from_scalable {
    ($($variant:ty),*) => {
        $(
            impl Scalable for $variant {
                fn scale<R: FromF64>(self, from_low: impl ToF64, from_high: impl ToF64, to_low: impl ToF64, to_high: impl ToF64) -> R {
                    let from_low = from_low.to_f64();
                    let from_high = from_high.to_f64();
                    let to_low = to_low.to_f64();
                    let to_high = to_high.to_f64();

                    let result = (self as f64 - from_low) * (to_high - to_low) / (from_high - from_low) + to_low;

                    R::from_f64(result)
                }
            }

            impl ToF64 for $variant {
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }

            impl FromF64 for $variant {
                fn from_f64(value: f64) -> Self {
                    value as $variant
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
    fn test_scale_unsigned() {
        assert_eq!(50.scale::<u8>(0, 100, 0, 255), 127);
        assert_eq!(0.scale::<u8>(0, 100, 0, 255), 0);
        assert_eq!(100.scale::<u8>(0, 100, 0, 255), 255);

        assert_eq!(0.scale::<u16>(0, 100, 180, 0), 180);
        assert_eq!(0.75.scale::<u16>(0, 1, 0, 180), 135);
        assert_eq!(0.75.scale::<u16>(0, 1, 180, 0), 45);
        assert_eq!(100.scale::<u16>(0, 100, 180, 0), 0);

        assert_eq!(0.scale::<u32>(0, 100, 180, 0), 180);
        assert_eq!(0.75.scale::<u32>(0, 1, 0, 180), 135);
        assert_eq!(0.75.scale::<u32>(0, 1, 180, 0), 45);
        assert_eq!(100.scale::<u32>(0, 100, 180, 0), 0);

        assert_eq!(0.scale::<u64>(100, 0, 180, 0), 0);
        assert_eq!(0.75.scale::<u64>(1, 0, 0, 180), 45);
        assert_eq!(0.75.scale::<u64>(1, 0, 180, 0), 135);
        assert_eq!(100.scale::<u64>(100, 0, 180, 0), 180);
    }

    #[test]
    fn test_scale_signed() {
        assert_eq!(50.scale::<i8>(0, 100, -50, 50), 0);
        assert_eq!(0.scale::<i8>(0, 100, -50, 0), -50);
        assert_eq!(100.scale::<i8>(0, 100, -50, 50), 50);

        assert_eq!(0.scale::<i16>(0, 100, 180, 0), 180);
        assert_eq!(-0.75.scale::<i16>(0, -1, 0, 180), 135);
        assert_eq!(-0.25.scale::<i16>(-1, 0, 180, 0), 45);
        assert_eq!(100.scale::<i16>(0, 100, 180, 0), 0);

        assert_eq!(0.scale::<i32>(0, 100, 180, 0), 180);
        assert_eq!(0.75.scale::<i32>(0, 1, 0, 180), 135);
        assert_eq!(0.75.scale::<i32>(0, 1, 180, 0), 45);
        assert_eq!(100.scale::<i32>(0, 100, 180, 0), 0);

        assert_eq!(0.scale::<i64>(100, 0, 180, 0), 0);
        assert_eq!(0.75.scale::<i64>(1, 0, 0, 180), 45);
        assert_eq!(0.75.scale::<i64>(1, 0, 180, 0), 135);
        assert_eq!(100.scale::<i64>(100, 0, 180, 0), 180);
    }

    #[test]
    fn test_scale_float() {
        assert!((0.5.scale::<f32>(0, 1, 0, 100) - 50.0).abs() < f32::EPSILON);
        assert!((0.scale::<f32>(0, 1, 0, 100) - 0.0).abs() < f32::EPSILON);
        assert!((1.scale::<f32>(0, 1, 0, 100) - 100.0).abs() < f32::EPSILON);

        assert_eq!(0.scale::<f32>(0, 100, 180, 0), 180.0);
        assert_eq!(0.75.scale::<f32>(0, 1, 0, 180), 135.0);
        assert_eq!(0.75.scale::<f32>(0, 1, 180, 0), 45.0);
        assert_eq!(100.scale::<f32>(0, 100, 180, 0), 0.0);

        assert!((0.5.scale::<f64>(0, 1, 0, 100) - 50.0).abs() < f64::EPSILON);
        assert!((0.scale::<f64>(0, 1, 0, 100) - 0.0).abs() < f64::EPSILON);
        assert!((1.scale::<f64>(0, 1, 0, 100) - 100.0).abs() < f64::EPSILON);

        assert_eq!(0.scale::<f64>(0, 100, 180, 0), 180.0);
        assert_eq!(0.75.scale::<f64>(0, 1, 0, 180), 135.0);
        assert_eq!(0.75.scale::<f64>(0, 1, 180, 0), 45.0);
        assert_eq!(100.scale::<f64>(0, 100, 180, 0), 0.0);
    }
}
