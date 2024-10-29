use crate::animations::Easing;
use crate::utils::Scalable;
use crate::utils::State;

/// Represents a keyframe in an animation sequence.
///
/// A `Keyframe` specifies a target value to be applied to the [`Output`](crate::devices::Output) of the
/// [`Track`](crate::animations::Track) to which this keyframe belongs. The [`Output`](crate::devices::Output)'s state will be
/// smoothly transitioned from its current state to the target value during the animation.
/// This transition occurs between the `start` timestamp and the `end` timestamp.
///
/// During this period, a smooth `transition` is applied using an [`Easing`] function,
/// which controls how the value changes from the current state to the target state.
///
/// # Example
///
/// If a `Keyframe` is set with a target value of 100, a start time of 0 ms, and an end time of 1000 ms,
/// the `Output`'s value will gradually move towards value 100 (whatever it means to it: let it
/// be the brightness of a LED, or the position of a Servo), over 1000 milliseconds, following the
/// defined easing function.
/// ```
/// use hermes_five::animations::{Easing, Keyframe};
/// let keyframe = Keyframe::new(100, 0, 1000).set_transition(Easing::SineInOut);
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// The target value of the keyframe: will be applied as a state for the [`Output`](crate::devices::Output) of the
    /// [`Track`] this keyframe belong to.
    target: State,
    /// The start time of the keyframe in milliseconds.
    start: u64,
    /// The end time of the keyframe in milliseconds.
    end: u64,
    /// The easing function applied during the transition (default: `Easing::Linear`).
    transition: Easing,
}

impl Keyframe {
    /// Creates a new `Keyframe` with the specified target value, start, and end times.<br/>
    /// _default `Easing::linear` function is used.
    ///
    /// # Arguments
    /// * `target` - The target state value.
    /// * `start` - The start time of the keyframe in milliseconds.
    /// * `end` - The end time of the keyframe in milliseconds.
    ///
    /// # Panic
    /// Panics if timestamps order are wrong: end < start.
    ///
    /// # Example
    /// ```
    /// use hermes_five::animations::Keyframe;
    /// let keyframe = Keyframe::new(100, 0, 1000);
    /// ```
    pub fn new<S: Into<State>>(target: S, start: u64, end: u64) -> Keyframe {
        assert!(
            start <= end,
            "Start time must be less than or equal to end time."
        );

        Keyframe {
            target: target.into(),
            start,
            end,
            transition: Easing::default(),
        }
    }

    /// Returns the duration of the keyframe.
    pub fn get_duration(&self) -> u64 {
        self.end - self.start
    }

    /// Computes the coefficient of the target value at a given time.
    ///
    /// This function calculates the progress of the current time relative to the keyframe's duration,
    /// clamping the time between the `start` and `end` timestamps. It then applies the easing function
    /// to this progress to determine the coefficient (fraction) of the target value that should be applied.
    ///
    /// In other words, the returned coefficient (ranging from 0.0 to 1.0) represents the percentage of the
    /// transition from the previous state to the target state at a given time. This coefficient indicates
    /// how far the transition has progressed at that time.
    ///
    /// The actual value represented by this coefficient depends on both the target state and the previous
    /// state, which is why the keyframe itself cannot compute the final value. The keyframe only provides
    /// the coefficient, which the [`Track`] will use to determine the correct value of the device state
    /// at the given time.
    ///
    /// # Arguments
    /// * `time` - The current time in milliseconds, which will be clamped between `start` and `end` timestamps.
    ///
    /// # Returns
    /// A coefficient between 0.0 and 1.0 that represents the fraction of the target value to be applied
    /// at the given time. For instance, if the coefficient is 0.75, it means that at the given time,
    /// 75% of the target value should be applied, considering the easing function.
    ///
    /// # Example
    /// If a keyframe has a target value of 100, a start time of 0 ms, and an end time of 1000 ms,
    /// and the easing function results in a coefficient of 0.75 at 600 ms, the output would be 0.75.
    /// This means that 75% of the target value transition (from previous keyframe target to 100)
    /// should be applied at that time.
    /// ```ignore
    /// use hermes_five::animations::{Easing, Keyframe};
    /// let keyframe = Keyframe::new(100, 0, 1000).set_transition(Easing::QuadOut);
    /// assert_eq!(keyframe.compute_target_coefficient(500), 0.75);
    /// ```
    pub(crate) fn compute_target_coefficient(&self, time: u64) -> f32 {
        let clamped_time = time.clamp(self.start, self.end) as f32;
        let progress = clamped_time.scale(self.start as f32, self.end as f32, 0.0, 1.0);
        self.transition.call(progress)
    }

    /// Returns  the target state for the keyframe.
    pub fn get_target(&self) -> State {
        self.target.clone()
    }

    /// Returns  the start time of the keyframe.
    pub fn get_start(&self) -> u64 {
        self.start
    }

    /// Returns  the end time of the keyframe.
    pub fn get_end(&self) -> u64 {
        self.end
    }

    /// Returns  the easing function used in the keyframe.
    pub fn get_transition(&self) -> Easing {
        self.transition
    }

    /// Sets a new easing function for the keyframe.
    pub fn set_transition(mut self, transition: Easing) -> Self {
        self.transition = transition;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_new() {
        let keyframe = Keyframe::new(100, 0, 1000);
        assert_eq!(keyframe.get_target().as_integer(), 100);
        assert_eq!(keyframe.get_start(), 0);
        assert_eq!(keyframe.get_end(), 1000);
        assert_eq!(keyframe.get_duration(), 1000);
        let keyframe = keyframe.set_transition(Easing::QuadOut);
        assert_eq!(keyframe.get_transition(), Easing::QuadOut);
    }

    #[test]
    #[should_panic(expected = "Start time must be less than or equal to end time.")]
    fn test_new_panic_start_greater_than_end() {
        // This test should panic because start is greater than end
        let _ = Keyframe::new(100, 2000, 1000);
    }

    #[test]
    fn test_keyframe_duration() {
        let keyframe = Keyframe::new(100, 1000, 2000);
        assert_eq!(keyframe.get_duration(), 1000);
    }

    #[test]
    fn test_compute_target_coefficient() {
        let keyframe = Keyframe::new(100, 0, 1000);
        let progress = keyframe.compute_target_coefficient(500);
        assert_eq!(progress, 0.5); // Assuming Easing::default() is linear scaling from 0.0 to 1.0
        let keyframe = keyframe.set_transition(Easing::QuadOut);
        let progress = keyframe.compute_target_coefficient(500);
        assert_eq!(progress, 0.75);

        // Assuming Easing::default() is linear scaling from 0.0 to 1.0:
        let keyframe = Keyframe::new(100, 1000, 2000);
        assert_eq!(keyframe.compute_target_coefficient(500), 0.0); // 0% if time is before start
        assert_eq!(keyframe.compute_target_coefficient(2500), 1.0); // 100% if time is after end
        assert_eq!(keyframe.compute_target_coefficient(1300), 0.3); // 30% if time is 30% of start to end
    }
}
