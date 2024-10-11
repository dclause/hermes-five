use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use crate::animation::Track;
use crate::errors::Error;
use crate::pause;

/// Represents an [`Animation`] unit, called a `Segment`.
///
/// A `Segment` is composed of multiple [`Track`]s, each containing sets of [`Keyframe`] associated with an [`Actuator`].
///
/// The `Segment` plays the keyframes of its track in a logical and temporal order.
/// - A segment searches for keyframe to execute and updates the associated devices based on a given
/// number of times per second, as defined by its `fps` property.
/// - A segment can be set to repeat in a loop within an [`Animation`] from a starting point in time called `loopback`.
///
/// # Example
///
/// Here is an example of defining a segment to animate a small robot with two actuators (a LED and a servo).
/// The robot will perform a waving motion using its servo and LED.
/// ```no_run
/// use hermes_five::animation::{Keyframe, Segment, Track};
/// use hermes_five::Board;
/// use hermes_five::devices::{Led, Servo};
/// use hermes_five::protocols::SerialProtocol;
/// use hermes_five::utils::Easing;
///
/// #[hermes_five::runtime]
/// async fn main() {
///     // Define a board on COM4.
///     let board = Board::from(SerialProtocol::new("COM4")).open();
///
///     // Define a servo attached to the board on PIN 9 (default servo position is 90°).
///     let servo = Servo::new(&board, 9, 90).unwrap();
///     // Create a track for the servo.
///     let servo_track = Track::new(servo)
///         // Turns the servo to 180° in 1000ms
///         .with_keyframe(Keyframe::new(180, 0, 1000).set_transition(Easing::SineInOut))
///         // Turns the servo to 0° in 1000ms
///         .with_keyframe(Keyframe::new(0, 2000, 3000).set_transition(Easing::SineInOut));
///
///     // Define a LED attached to the board on PIN 13.
///     let led = Led::new(&board, 13).unwrap();
///     // Create a track for the LED.
///     let led_track = Track::new(led)
///         // Turns the LED fully (instantly)
///         .with_keyframe(Keyframe::new(255, 0, 1))
///         // Fade out the LED in 1000ms
///         .with_keyframe(Keyframe::new(0, 2000, 3000));
///
///     // Create an animation Segment for this:
///     let segment = Segment::default()
///         .with_track(servo_track)
///         .with_track(led_track)
///         .set_repeat(true);
/// }
/// ```
///
/// # Fields
///
/// - `repeat`: Determines whether the segment should replay in a loop starting from the `loopback` time (default: false).
/// - `loopback`: The time in milliseconds when the animation will restart the loop if `repeat` is true (default: 0).
/// - `speed`: Controls the speed of the animation as a percentage of standard time (default: 100). For example:
///     - 50% means time moves twice as slow, so 1000ms lasts 2000ms in real time.
///     - 200% means time moves twice as fast, so 1000ms lasts 500ms in real time.
/// - `fps`: The number of frames per second for running the animation (default: 40). Higher fps results in smoother animations, though the desired fps is not always guaranteed to be reached (especially at high fps values).
/// - `tracks`: The tracks associated with this segment.
/// - `current_time`: The current time in milliseconds of the segment's playback.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Segment {
    /// Determines whether the segment should replay in a loop (starting from the [`Segment::loopback`] time).
    repeat: bool,
    /// The point in time (in ms) the animation will restart the loop when `loop` is set to true (default: 0).
    loopback: u64,
    /// Controls the speed of the animation. (default: 100)
    /// Defined as a percentage of standard time. For example:
    /// - 50% means time moves twice as slow, so 1000ms lasts 2000ms in real time.
    /// - 200% means time moves twice as fast, so 1000ms lasts 500ms in real time
    speed: u8,
    /// The number of frames per second (fps) for running the animation (default: 40fps).
    /// - Higher fps results in smoother animations.
    /// - Desired `fps` is not guaranteed to be reached (specially high fps values).
    /// - The `fps` can be overridden for each [`Segment`] in the animation.
    fps: u8,
    /// The tracks for this segment.
    tracks: Vec<Track>,

    // ########################################
    // # Volatile utility data.
    /// The current time (in ms) the segment is currently at.
    #[cfg_attr(feature = "serde", serde(skip))]
    current_time: u64,
}

impl From<Track> for Segment {
    fn from(track: Track) -> Self {
        Self::default().with_track(track)
    }
}

impl Segment {
    /// Plays the segment. If `repeat` is true, the segment will loop indefinitely.
    pub async fn play(&mut self) -> Result<(), Error> {
        if self.get_duration() > 0 {
            match self.is_repeat() {
                true => loop {
                    self.play_once().await?;
                    self.current_time = self.loopback;
                },
                false => self.play_once().await?,
            };
        }

        self.reset();
        Ok(())
    }

    /// Plays all tracks once only.
    pub(crate) async fn play_once(&mut self) -> Result<(), Error> {
        let start_time = SystemTime::now();

        let total_duration = self.get_duration();
        // The theoretical time a frame should take is defined by the `fps` settings for the segment.
        // This is theoretical since this is the minimum amount of time we want a single timeframe to last.
        // But it may take more time if the segment have to run multiple tracks / keyframes for this timeframe.
        let theoretical_timeframe_duration = 1000u64 / self.fps as u64;
        // The realtime a frame took: at this point this is 0ms, but we will measure that in the following.
        let mut realtime_timeframe_duration;

        // As long as we did not reach the segment duration, we know we have some work to do.
        while self.current_time < total_duration {
            // Take a measure of time here (will be used to measure realtime_timeframe_duration)
            let realtime_start = SystemTime::now();

            // The timeframe starts now (current_time).
            // It ends, in theory, after a realtime_timeframe_duration amount of time.
            // However, if realtime_timeframe_duration was longer on the last timeframe, we can
            // anticipate it to be longer this time too.
            // let timeframe_end =
            //     self.current_time + theoretical_timeframe_duration.max(realtime_timeframe_duration);

            // Ask each track to play the timeframe.
            for track in &mut self.tracks {
                track.play_frame([
                    self.current_time,
                    self.current_time + theoretical_timeframe_duration,
                ])?;
            }

            // Here we can know how long actually took the timeframe.
            let realtime_end = SystemTime::now();
            realtime_timeframe_duration = realtime_end
                .duration_since(realtime_start)
                .unwrap()
                .as_millis() as u64;

            // If the timeframe took less time than expected: we need to wait.
            if realtime_timeframe_duration < theoretical_timeframe_duration {
                pause!(theoretical_timeframe_duration - realtime_timeframe_duration);
            }

            self.current_time = start_time.elapsed().unwrap().as_millis() as u64;
        }

        Ok(())
    }

    /// Resets the segment's current time to 0.
    pub(crate) fn reset(&mut self) {
        self.current_time = 0;
        /*for track in &mut self.tracks {
            track.get_device().as_mut().stop()
        }*/
    }

    /// Gets the total duration of the segment.
    ///
    /// The duration is determined by the longest track in the segment.
    pub fn get_duration(&self) -> u64 {
        match self.tracks.len() > 0 {
            false => 0,
            true => {
                let longest_track = self
                    .tracks
                    .iter()
                    .max_by(|x, y| x.get_duration().cmp(&y.get_duration()))
                    .unwrap();
                longest_track.get_duration()
            }
        }
    }

    /// Gets the current play time.
    pub fn get_progress(&self) -> u64 {
        self.current_time
    }
}

// ########################################
// Implementing basic getters and setters.
impl Segment {
    /// Checks if the segment should repeat.
    pub fn is_repeat(&self) -> bool {
        self.repeat
    }
    /// Returns the loopback time.
    pub fn get_loopback(&self) -> u64 {
        self.loopback
    }
    /// Returns the playback speed as a percentage (between 0% and 100%).
    pub fn get_speed(&self) -> u8 {
        self.speed
    }
    /// Returns the frames per second (fps) for the segment.
    pub fn get_fps(&self) -> u8 {
        self.fps
    }
    /// Returns the tracks in the segment.
    pub fn get_tracks(&self) -> &Vec<Track> {
        &self.tracks
    }

    /// Sets whether the segment should repeat and returns the updated segment.
    pub fn set_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
    /// Sets the loopback time and returns the updated segment.
    pub fn set_loopback(mut self, loopback: u64) -> Self {
        self.loopback = loopback;
        self
    }
    /// Sets the playback speed as a percentage (between 0% and 100%) and returns the updated segment.
    pub fn set_speed(mut self, speed: u8) -> Self {
        self.speed = speed;
        self
    }
    /// Sets the frames per second (fps) and returns the updated segment.
    pub fn set_fps(mut self, fps: u8) -> Self {
        self.fps = fps;
        self
    }
    /// Sets the tracks for the segment and returns the updated segment.
    pub fn set_tracks(mut self, tracks: Vec<Track>) -> Self {
        self.tracks = tracks;
        self
    }

    /// Adds a track to the segment and returns the updated segment.
    pub fn with_track(mut self, track: Track) -> Self {
        self.tracks.push(track);
        self
    }
}

impl Default for Segment {
    fn default() -> Self {
        Segment {
            repeat: false,
            loopback: 0,
            speed: 100,
            fps: 60,
            tracks: vec![],
            current_time: 0,
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment: {} tracks - duration: {}ms",
            self.tracks.len(),
            self.get_duration()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::animation::{Keyframe, Segment};
    use crate::animation::Track;
    use crate::mocks::actuator::MockActuator;

    #[test]
    fn test_segment_default() {
        let segment = Segment::default();
        assert!(!segment.is_repeat());
        assert_eq!(segment.get_loopback(), 0);
        assert_eq!(segment.get_speed(), 100);
        assert_eq!(segment.get_fps(), 60);
        assert!(segment.get_tracks().is_empty());
        assert_eq!(segment.get_duration(), 0);
    }

    #[test]
    fn test_segment_setters() {
        let segment = Segment::default()
            .set_repeat(true)
            .set_loopback(100)
            .set_speed(150)
            .set_fps(100)
            .set_tracks(vec![
                Track::new(MockActuator::new(50)),
                Track::new(MockActuator::new(100)),
            ]);

        assert!(segment.is_repeat());
        assert_eq!(segment.get_loopback(), 100);
        assert_eq!(segment.get_speed(), 150);
        assert_eq!(segment.get_fps(), 100);
        assert_eq!(segment.get_tracks().len(), 2);
    }

    #[test]
    fn test_segment_reset() {
        let mut segment = Segment::default();
        segment.current_time = 100;
        segment.reset();
        assert_eq!(segment.get_progress(), 0);
    }

    #[test]
    fn test_segment_duration() {
        let segment = Segment::default().set_tracks(vec![
            Track::new(MockActuator::new(50))
                .with_keyframe(Keyframe::new(10, 0, 500))
                .with_keyframe(Keyframe::new(20, 600, 4000)),
            Track::new(MockActuator::new(100))
                .with_keyframe(Keyframe::new(10, 3000, 3300))
                .with_keyframe(Keyframe::new(20, 3500, 3800)),
        ]);

        assert_eq!(segment.get_duration(), 4000);
    }

    #[tokio::test]
    async fn test_segment_play_once() {
        let mut segment = Segment::default()
            .set_tracks(vec![
                Track::new(MockActuator::new(50))
                    .with_keyframe(Keyframe::new(10, 0, 100))
                    .with_keyframe(Keyframe::new(20, 200, 300)),
                Track::new(MockActuator::new(100)).with_keyframe(Keyframe::new(10, 300, 500)),
            ])
            .set_fps(100);

        assert_eq!(segment.get_progress(), 0);
        let start = SystemTime::now();
        let play_once = segment.play_once().await;
        let elapsed = start.elapsed().unwrap().as_millis();
        assert!(play_once.is_ok());
        assert!(
            elapsed >= 500 && elapsed < 550,
            "Play once takes longer approx. the time of the longest track: {}",
            elapsed
        );
        assert!(segment.get_progress() >= 500)
    }

    #[tokio::test]
    async fn test_segment_play() {
        let mut segment = Segment::default()
            .set_tracks(vec![
                Track::new(MockActuator::new(50)).with_keyframe(Keyframe::new(10, 0, 100))
            ])
            .set_fps(100);

        // ########################################
        // Play no-repeat

        let start = SystemTime::now();
        let play = segment.play().await;
        let elapsed = start.elapsed().unwrap().as_millis();
        assert!(play.is_ok());
        assert!(
            elapsed >= 100 && elapsed < 150,
            "Play takes the same time as play once: {}",
            elapsed
        );

        // ########################################
        // Play repeat

        let mut segment = segment.set_repeat(true);

        tokio::select! {
            _ = segment.play() => assert!(false, "Infinite play should not finish first"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => assert!(true, "Infinite play is infinite")
        }
    }

    #[test]
    fn test_track_to_segment() {
        let segment = Segment::from(Track::new(MockActuator::new(50)));
        assert_eq!(segment.get_tracks().len(), 1);
    }

    #[test]
    fn test_display_implementation() {
        let segment = Segment::default()
            .with_track(
                Track::new(MockActuator::new(50))
                    .with_keyframe(Keyframe::new(10, 0, 500))
                    .with_keyframe(Keyframe::new(20, 600, 4000)),
            )
            .with_track(
                Track::new(MockActuator::new(100))
                    .with_keyframe(Keyframe::new(10, 3000, 3300))
                    .with_keyframe(Keyframe::new(20, 3500, 3800)),
            );

        let expected_display = "Segment: 2 tracks - duration: 4000ms";
        assert_eq!(format!("{}", segment), expected_display);
    }
}
