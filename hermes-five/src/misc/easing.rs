use simple_easing::*;

/// Represents a set of easing method.
///
/// An easing function is a temporal function that takes a time between 0 and 1 (beginning / end)
/// and associate to it a number value according to an ease curve.
///
/// see https://easings.net for a representation of easing methods
pub enum Easing {
    Linear,
    SinIn,
}

impl Easing {
    /// Call the easing function.
    /// ```
    /// fn apply_easing(easing: Easing, t: f32) -> f32 {
    ///     easing.call(t)
    /// }
    /// ````
    pub(crate) fn call(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => linear(t),
            Easing::SinIn => sine_in(t),
        }
    }
}
