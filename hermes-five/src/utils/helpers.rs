use std::ops::{Add, Div, Mul, Sub};

/// Trait for mapping a value from one range to another.
pub trait MapRange<T> {
    /// Maps a value from one range to another.
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
    fn map(self, from_low: T, from_high: T, to_low: T, to_high: T) -> T;
}

impl<T> MapRange<T> for T
where
    T: Copy + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Add<Output = T>,
{
    fn map(self, from_low: T, from_high: T, to_low: T, to_high: T) -> T {
        (self - from_low) * (to_high - to_low) / (from_high - from_low) + to_low
    }
}
//
// impl<T: Into<u8>> MapRange<T> for PinValue {
//     fn map(self, from_low: T, from_high: T, to_low: T, to_high: T) -> T {
//         let value = self as u8;
//         let f_low = from_low as u8;
//         let f_high = from_high as u8;
//         let t_low = to_low as u8;
//         let t_high = to_high as u8;
//         PinValue::from_u8((value - f_low) * (t_high - t_low) / (f_high - f_low) + t_low).unwrap()
//     }
// }
