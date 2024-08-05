use std::fmt::{Display, Formatter};

use crate::animation::Keyframe;
use crate::devices::Actuator;
use crate::errors::Error;
use crate::utils::Range;
use crate::utils::scale::Scalable;

/// Represents an animation track within a [`Sequence`] for a given [`Actuator`].
///
/// The `Track` struct manages the state and keyframes for an actuator device through a sequence.
/// It represents the evolution of the device internal state over the sequence (animation) period
/// at specific `Keyframe` and the transition between those.
///
/// # Example
///
/// If a `Keyframe` is set with a target value of 100, a start time of 0 ms, and an end time of 1000 ms,
/// the `Actuator`'s value will gradually move towards value 100 (whatever it means to it: let it
/// be the brightness of a LED, or the position of a Servo), over 1000 milliseconds, following the
/// defined easing function.
/// ```no_run
/// use hermes_five::animation::{Keyframe, Track};
/// use hermes_five::Board;
/// use hermes_five::devices::Servo;
/// use hermes_five::protocols::SerialProtocol;
/// use hermes_five::utils::Easing;
///
/// #[hermes_five::runtime]
/// async fn main() {
///     // Defines a board (using serial port on COM4).
///     let board = Board::from(SerialProtocol::new("COM4")).open();
///     // Defines a servo attached to the board on PIN 9 (default servo position is 90°).
///     let servo = Servo::new(&board, 9, 90).unwrap();
///     // Creates a track for the servo.
///     let track = Track::new(servo)
///         // Turns the servo to 180° in 1000ms
///         .with_keyframe(Keyframe::new(180, 0, 1000).set_transition(Easing::SineInOut))
///         // Turns the servo to 0° in 1000ms
///         .with_keyframe(Keyframe::new(0, 2000, 3000).set_transition(Easing::SineInOut));
/// }
/// ```
///
/// # Fields
///
/// * `name`: The name of the track.
/// * `device`: The [`Actuator`] associated with this track.
/// * `keyframes`: A list of [`Keyframe`]s defining the animation.
/// * `previous`: The previous state value of the actuator.
/// * `current`: The current state value of the actuator.
#[derive(Clone, Debug)]
pub struct Track {
    // @todo keep?
    name: String,
    /// The [`Actuator`] device that this track is associated with.
    /// All keyframes' [`Keyframe::target`] values will reference this device.
    device: Box<dyn Actuator>,
    /// The [`Keyframe`]s belonging to this track.
    keyframes: Vec<Keyframe>,

    // (Internal): keyframe history
    previous: u16,
    current: u16,
}

impl Track {
    /// Creates a new `Track` associated with the given actuator.
    ///
    /// # Arguments
    /// * `device` - The actuator device this track will control.
    ///
    /// # Returns
    /// A new `Track` instance with the provided actuator and an empty list of keyframes.
    #[allow(private_bounds)]
    pub fn new<T: Actuator + 'static>(device: T) -> Self {
        let history = device.get_state();
        Self {
            name: String::from("New track"),
            device: Box::new(device),
            keyframes: vec![],
            previous: history,
            current: history,
        }
    }

    /// Compute and return the total duration of the track, which is the end time of the last keyframe.
    ///
    /// # Returns
    /// The duration in milliseconds (0 if there are no keyframes).
    pub fn get_duration(&self) -> u64 {
        match self.keyframes.len() > 0 {
            false => 0,
            true => {
                let last_keyframe = self
                    .keyframes
                    .iter()
                    .max_by(|x, y| x.get_end().cmp(&(y.get_end() as u64)))
                    .unwrap();
                last_keyframe.get_end()
            }
        }
    }

    /// Plays the keyframes within the given timeframe, updating the actuator state accordingly.
    ///
    /// # Arguments
    /// * `timeframe` - A range of timestamps to consider for keyframe playback.
    ///
    /// # Returns
    /// A result indicating success or an error if the state update fails.
    pub(crate) fn play_frame<F: Into<Range<u64>>>(&mut self, timeframe: F) -> Result<(), Error> {
        let timeframe = timeframe.into();
        // Get the keyframe to be played according to the time frame.
        let keyframe = self.get_best_keyframe(timeframe);

        match keyframe {
            None => Ok(()),
            Some(keyframe) => {
                self.update_history(keyframe.get_target());
                let progress = keyframe.compute_target_coefficient(timeframe.end);
                let value: u16 = progress.scale(0, 1, self.previous, keyframe.get_target());
                self.device.set_state(value)
            }
        }
    }

    /// Finds the most appropriate keyframe for the given timeframe.
    ///
    /// The strategy is to find all keyframes which start-end period intersect with the timeframe
    /// and return the last ending one.
    /// The reason behind it is that no keyframe should overlap in theory (does not make sense) on
    /// a same track, so if it does, the last ending one is the longest, hence the one for which the
    /// transition will be the stablest over time.
    /// ```
    /// // --|---------------|------------> time
    /// //   |  ####         |
    /// //   |    ###########|##  this one will lead to the stabled values overtime
    /// // ##|####           |
    /// ```
    /// # Arguments
    /// * `timeframe` - The range of timestamps to find a matching keyframe for.
    ///
    /// # Returns
    /// An `Option` containing the most relevant `Keyframe` for the given timeframe, if any.
    fn get_best_keyframe<'a, R: Into<Range<u64>>>(&mut self, timeframe: R) -> Option<Keyframe> {
        let timeframe = timeframe.into();
        // Get the keyframe to be played: the last one that
        self.keyframes
            .iter()
            .filter(|kf| {
                // case keyframe starts during the interval
                kf.get_start() >= timeframe.start && kf.get_start() < timeframe.end ||
                // case keyframe ends during the interval
                kf.get_end() >= timeframe.start && kf.get_end() < timeframe.end ||
                // case keyframe is running during the interval
                kf.get_start() <= timeframe.start && kf.get_end() > timeframe.end
            })
            .max_by(|a, b| a.get_end().cmp(&b.get_end()))
            .cloned()
    }

    /// Updates the internal state history with the new keyframe's target value if it has changed.
    ///
    /// # Arguments
    /// * `new_state` - The new state to be added in history.
    fn update_history(&mut self, new_state: u16) {
        if self.current != new_state {
            self.previous = self.current;
            self.current = new_state;
        }
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Track '{}': {} keyframes - duration: {}ms",
            self.name,
            self.keyframes.len(),
            self.get_duration()
        )
    }
}

// ########################################
// Implementing basic getters and setters.
impl Track {
    /// Returns the name for the [`Track`].
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    /// Returns the device associated with the [`Track`].
    #[allow(private_interfaces)]
    pub fn get_device(&self) -> &Box<dyn Actuator> {
        &self.device
    }
    /// Returns the keyframes of this [`Track`].
    pub fn get_keyframes(&self) -> &Vec<Keyframe> {
        &self.keyframes
    }

    /// Returns the keyframes of this [`Track`].
    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    /// Add a new keyframe to this [`Track`].
    ///
    /// No validation is done on keyframe validity: at any moment, only one keyframe (the best
    /// suitable one according to [`Track::get_best_keyframe()`] will be played.
    /// So some keyframes may be missed if overlapping for instance.
    pub fn with_keyframe(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::Keyframe;
    use crate::mocks::actuator::MockActuator;
    use crate::utils::Range;

    use super::*;

    #[test]
    fn test_new_track() {
        let actuator = MockActuator::new(5);
        let track = Track::new(actuator);

        assert_eq!(track.get_name(), "New track");
        assert_eq!(track.get_keyframes().len(), 0);
        assert_eq!(track.previous, 5);
        assert_eq!(track.current, 5);

        let track = track
            .with_keyframe(Keyframe::new(50, 0, 2000))
            .set_name("New name");
        assert_eq!(track.get_name(), "New name");
        assert_eq!(track.get_keyframes().len(), 1);
    }

    #[test]
    fn test_get_duration() {
        let actuator = MockActuator::new(5);
        let track = Track::new(actuator);

        assert_eq!(
            track.get_duration(),
            0,
            "Track with no keyframe have a 0 duration."
        );

        let track = track
            .with_keyframe(Keyframe::new(50, 0, 2000))
            .with_keyframe(Keyframe::new(100, 500, 2200))
            .with_keyframe(Keyframe::new(100, 100, 1000));
        assert_eq!(
            track.get_duration(),
            2200,
            "Track duration is the end time of the latest keyframe."
        );
    }

    #[test]
    fn test_update_history() {
        let actuator = MockActuator::new(5);
        let mut track = Track::new(actuator);
        assert_eq!(track.previous, 5);
        assert_eq!(track.current, 5);

        track.update_history(75);

        assert_eq!(track.previous, 5); // Initial state was 5
        assert_eq!(track.current, 75); // updated to 75

        track.update_history(100);
        assert_eq!(track.previous, 75); // Previous update was 75
        assert_eq!(track.current, 100); // updated to 100
    }

    #[test]
    fn test_get_best_keyframe() {
        let mut track = Track::new(MockActuator::new(100))
            .with_keyframe(Keyframe::new(60, 0, 2000))
            .with_keyframe(Keyframe::new(70, 500, 2200))
            .with_keyframe(Keyframe::new(80, 100, 2100));

        let keyframe = track.get_best_keyframe([0, 100]);
        assert!(keyframe.is_some());
        assert_eq!(keyframe.unwrap().get_target(), 60);

        let keyframe = track.get_best_keyframe([300, 400]);
        assert!(keyframe.is_some());
        assert_eq!(keyframe.unwrap().get_target(), 80);

        let keyframe = track.get_best_keyframe([600, 800]);
        assert!(keyframe.is_some());
        assert_eq!(keyframe.unwrap().get_target(), 70);

        let keyframe = track.get_best_keyframe([3000, 3200]);
        assert!(keyframe.is_none());
    }

    #[test]
    fn test_play_frame_no_keyframes() {
        let actuator = MockActuator::new(5);
        let mut track = Track::new(actuator);

        let result = track.play_frame(Range {
            start: 0,
            end: 1000,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_frame() {
        let actuator = MockActuator::new(0);
        let mut track = Track::new(actuator);

        // Don't fail with no keyframe.
        let result = track.play_frame([500, 1500]);
        assert!(result.is_ok());
        assert_eq!(track.previous, 0);
        assert_eq!(track.current, 0);

        // Play a within a timeframe updates the history and the device accordingly.
        let mut track = track
            .with_keyframe(Keyframe::new(50, 0, 2000))
            .with_keyframe(Keyframe::new(70, 500, 2500))
            .with_keyframe(Keyframe::new(90, 100, 1000));

        let result = track.play_frame([500, 1500]); // 1500ms is the middle of the second keyframe.
        assert!(result.is_ok());
        assert_eq!(track.previous, 0);
        assert_eq!(track.current, 70); // Second keyframe target is 70
        assert_eq!(track.get_device().get_state(), 35); // But at 50% it has a 70/2=35 value (Easing::Linear by default)
    }

    #[test]
    fn test_display_implementation() {
        let track = Track::new(MockActuator::new(5))
            .with_keyframe(Keyframe::new(50, 0, 2000))
            .with_keyframe(Keyframe::new(100, 500, 2200))
            .with_keyframe(Keyframe::new(100, 100, 1000));

        let expected_display = "Track 'New track': 3 keyframes - duration: 2200ms";
        assert_eq!(format!("{}", track), expected_display);
    }
}
