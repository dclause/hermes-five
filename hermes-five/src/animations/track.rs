use std::fmt::{Display, Formatter};

use crate::animations::Keyframe;
use crate::devices::Output;
use crate::errors::Error;
use crate::utils::{Range, State};

/// Represents an animation track within a [`Segment`](crate::animations::Segment) for a given [`Output`](Output) device.
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
/// use hermes_five::animations::{Easing, Keyframe, Track};
/// use hermes_five::hardware::Board;
/// use hermes_five::devices::Servo;
/// use hermes_five::io::RemoteIo;
///
/// #[hermes_five::runtime]
/// async fn main() {
///     // Defines a board (using serial port on COM4).
///     let board = Board::new(RemoteIo::new("COM4")).open();
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Track {
    /// The [`Output`] device that this track is associated with.
    /// All keyframes [`Keyframe::target`] values will reference this device.
    device: Box<dyn Output>,
    /// The [`Keyframe`]s belonging to this track.
    keyframes: Vec<Keyframe>,

    // ########################################
    // # Volatile utility data.
    #[cfg_attr(feature = "serde", serde(skip))]
    previous: State,
    #[cfg_attr(feature = "serde", serde(skip))]
    current: State,
}

impl Track {
    /// Creates a new `Track` associated with the given actuator.
    #[allow(private_bounds)]
    pub fn new<T: Output + 'static>(device: T) -> Self {
        let history = device.get_state();
        Self {
            device: Box::new(device),
            keyframes: vec![],
            previous: history.clone(),
            current: history,
        }
    }

    /// Compute and return the total duration (in ms) of the track. The duration is by definition the end time of the last keyframe.
    pub fn get_duration(&self) -> u64 {
        match !self.keyframes.is_empty() {
            false => 0,
            true => {
                let last_keyframe = self
                    .keyframes
                    .iter()
                    .max_by(|x, y| x.get_end().cmp(&(y.get_end())))
                    .unwrap();
                last_keyframe.get_end()
            }
        }
    }

    /// Plays the keyframes within the given timeframe, updating the actuator state accordingly.
    pub(crate) fn play_frame<F: Into<Range<u64>>>(&mut self, timeframe: F) -> Result<(), Error> {
        let timeframe = timeframe.into();
        // Get the keyframe to be played according to the time frame.
        let keyframe = self.get_best_keyframe(timeframe);

        match keyframe {
            None => (),
            Some(keyframe) => {
                self.update_history(keyframe.get_target());
                let progress = keyframe.compute_target_coefficient(timeframe.end);
                let state =
                    self.device
                        .scale_state(self.previous.clone(), keyframe.get_target(), progress);
                self.device.set_state(state)?;
            }
        };

        Ok(())
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
    /// //   |    ###########|##  this one will lead to the stablest values overtime
    /// // ##|####           |
    /// ```
    fn get_best_keyframe<R: Into<Range<u64>>>(&mut self, timeframe: R) -> Option<Keyframe> {
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
    fn update_history(&mut self, new_state: State) {
        if self.current != new_state {
            self.previous = self.current.clone();
            self.current = new_state;
        }
    }

    /// Returns the device associated with the [`Track`].
    pub fn get_device(&self) -> &dyn Output {
        &*self.device
    }
    /// Returns the keyframes of this [`Track`].
    pub fn get_keyframes(&self) -> &Vec<Keyframe> {
        &self.keyframes
    }

    #[allow(rustdoc::private_intra_doc_links)]
    /// Add a new keyframe to this [`Track`].
    ///
    /// No validation is done on keyframe validity: at any moment, only one keyframe (the best
    /// suitable one according to [`Track::get_best_keyframe()`] strategy) will be played.
    /// So some keyframes may be missed if overlapping for instance.
    pub fn with_keyframe(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        self
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Track: {} keyframes - duration: {}ms",
            self.keyframes.len(),
            self.get_duration()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::animations::Keyframe;
    use crate::mocks::output_device::MockOutputDevice;
    use crate::utils::Range;

    use super::*;

    #[test]
    fn test_new_track() {
        let actuator = MockOutputDevice::new(5);
        let track = Track::new(actuator);

        assert_eq!(track.get_keyframes().len(), 0);
        assert_eq!(track.previous.as_integer(), 5);
        assert_eq!(track.current.as_integer(), 5);

        let track = track.with_keyframe(Keyframe::new(50, 0, 2000));
        assert_eq!(track.get_keyframes().len(), 1);
    }

    #[test]
    fn test_get_duration() {
        let actuator = MockOutputDevice::new(5);
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
        let actuator = MockOutputDevice::new(5);
        let mut track = Track::new(actuator);
        assert_eq!(track.previous.as_integer(), 5);
        assert_eq!(track.current.as_integer(), 5);

        track.update_history(75.into());

        assert_eq!(track.previous.as_integer(), 5); // Initial state was 5
        assert_eq!(track.current.as_integer(), 75); // updated to 75

        track.update_history(100.into());
        assert_eq!(track.previous.as_integer(), 75); // Previous update was 75
        assert_eq!(track.current.as_integer(), 100); // updated to 100
    }

    #[test]
    fn test_get_best_keyframe() {
        let mut track = Track::new(MockOutputDevice::new(100))
            .with_keyframe(Keyframe::new(60, 0, 2000))
            .with_keyframe(Keyframe::new(70, 500, 2200))
            .with_keyframe(Keyframe::new(80, 100, 2100));

        let keyframe = track.get_best_keyframe([0, 100]);
        assert!(keyframe.is_some());
        assert_eq!(keyframe.unwrap().get_target().as_integer(), 60);

        let keyframe = track.get_best_keyframe([300, 400]);
        assert!(keyframe.is_some());
        assert_eq!(keyframe.unwrap().get_target().as_integer(), 80);

        let keyframe = track.get_best_keyframe([600, 800]);
        assert!(keyframe.is_some());
        assert_eq!(keyframe.unwrap().get_target().as_integer(), 70);

        let keyframe = track.get_best_keyframe([3000, 3200]);
        assert!(keyframe.is_none());
    }

    #[test]
    fn test_play_frame_no_keyframes() {
        let actuator = MockOutputDevice::new(5);
        let mut track = Track::new(actuator);

        let result = track.play_frame(Range {
            start: 0,
            end: 1000,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_frame() {
        let actuator = MockOutputDevice::new(0);
        let mut track = Track::new(actuator);

        // Don't fail with no keyframe.
        let result = track.play_frame([500, 1500]);
        assert!(result.is_ok());
        assert_eq!(track.previous.as_integer(), 0);
        assert_eq!(track.current.as_integer(), 0);

        // Play a within a timeframe updates the history and the device accordingly.
        let mut track = track
            .with_keyframe(Keyframe::new(50, 0, 2000))
            .with_keyframe(Keyframe::new(70, 500, 2500))
            .with_keyframe(Keyframe::new(90, 100, 1000));

        let result = track.play_frame([500, 1500]); // 1500ms is the middle of the second keyframe.
        assert!(result.is_ok());
        assert_eq!(track.previous.as_integer(), 0);
        assert_eq!(track.current.as_integer(), 70); // Second keyframe target is 70
        assert_eq!(track.get_device().get_state().as_integer(), 35); // But at 50% it has a 70/2=35 value (Easing::Linear by default)
    }

    #[test]
    fn test_display_implementation() {
        let track = Track::new(MockOutputDevice::new(5))
            .with_keyframe(Keyframe::new(50, 0, 2000))
            .with_keyframe(Keyframe::new(100, 500, 2200))
            .with_keyframe(Keyframe::new(100, 100, 1000));

        let expected_display = "Track: 3 keyframes - duration: 2200ms";
        assert_eq!(format!("{}", track), expected_display);
    }
}
