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

impl MapRange<u16> for u16 {
    fn map(self, from_low: u16, from_high: u16, to_low: u16, to_high: u16) -> u16 {
        ((self as f64 - from_low as f64) * (to_high as f64 - to_low as f64)
            / (from_high as f64 - from_low as f64)
            + to_low as f64) as u16
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
