use std::sync::Arc;

use parking_lot::RwLock;

use crate::animation::{Segment, Track};
use crate::utils::task;
use crate::utils::task::TaskHandler;

/// Represents an animation: a collection of ordered [`Segment`]s to be run in sequence.
///
/// An animation can be played, paused, resumed, and stopped. Each [`Segment`] in the animation is
/// played in order (eventually looped, see [`Segment`]) until all are done.
///
/// # Example
///
/// Here is an example of an animation made of a single segment that sweeps a servo indefinitely,
/// from position 0° to 180° and back:
/// (sidenote: prefer use [`Servo::sweep()`] helper for this purpose).
/// ```
/// use hermes_five::pause;
/// use hermes_five::animation::{Animation, Keyframe, Segment, Track};
/// use hermes_five::Board;
/// use hermes_five::BoardEvent;
/// use hermes_five::devices::Servo;
/// use hermes_five::utils::Easing;
///
/// #[hermes_five::runtime]
/// async fn main() {
///     let board = Board::run();
///     board.on(BoardEvent::OnReady, |board: Board| async move {
///         let servo = Servo::new(&board, 9, 0).unwrap();
///
///         let mut animation = Animation::from(
///             Segment::from(
///                 Track::new(servo)
///                     .with_keyframe(Keyframe::new(180, 0, 500).set_transition(Easing::SineInOut))
///                     .with_keyframe(Keyframe::new(90, 1000, 2000).set_transition(Easing::SineInOut)),
///             )
///             .set_fps(100)
///             .set_repeat(true)
///         );
///
///         animation.play();
///         pause!(3000);
///
///         animation.stop();
///
///         Ok(())
///     });
/// }
/// ```
///
/// # Fields
///
/// - `segments`: The ordered list of animation [`Segment`]s.
/// - `current`: The index of the currently running [`Segment`] (starting at 0).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Animation {
    /// The ordered list of animation [`Segment`].
    segments: Vec<Segment>,

    // ########################################
    // # Volatile utility data.
    /// The index of current running [`Segment`].
    #[cfg_attr(feature = "serde", serde(skip))]
    current: Arc<RwLock<usize>>,
    /// Inner handler to the task running the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    interval: Arc<RwLock<Option<TaskHandler>>>,
}

// ########################################

impl Animation {
    /// Starts or resumes the animation.
    ///
    /// The animation will start from the current segment or from the beginning if it was stopped.
    pub fn play(&mut self) -> &mut Self {
        let mut self_clone = self.clone();
        let handler = task::run(async move {
            // Loop through the segments and run them one by one.
            for index in self_clone.get_current()..self_clone.segments.len() {
                *self_clone.current.write() = index;

                // Retrieve the currently running segment.
                let segment_playing = self_clone.segments.get_mut(index).unwrap();
                segment_playing.play().await?;
            }

            *self_clone.current.write() = 0; // reset to the beginning
            Ok(())
        })
        .unwrap();
        *self.interval.write() = Some(handler);

        self
    }

    /// Pauses the animation.
    ///
    /// When resumed, the animation will continue from the point it was paused.
    pub fn pause(&mut self) -> &mut Self {
        self.cancel_animation();
        self
    }

    /// Skips the current sequence and jump to next.
    ///
    /// Caveats:
    /// - Skipping the current segment does not pause / resume the animation: if it was running, it
    /// continues to do so (from the beginning of next segment).
    /// - If already on the last segment, the animation loop to the first one, hence, restart.
    pub fn next(&mut self) -> &mut Self {
        let current = self.get_current();

        // Stop the current animation (do not reuse `.stop()` to avoid triggering the stop event).
        let was_running = self.cancel_animation();

        // Reset current segment:
        if let Some(segment_playing) = self.segments.get_mut(current) {
            segment_playing.reset();
        }

        // Move to the next segment if we are not at the end.
        match current < self.segments.len() - 1 {
            true => *self.current.write() = current + 1,
            false => *self.current.write() = 0,
        }

        // Restart the animation from the beginning of the next segment, if it was running.
        if was_running {
            self.play();
        }

        self
    }

    /// Stops the animation.
    ///
    /// The animation will be reset to the beginning. The current segment is reset, and the index is set to 0.
    pub fn stop(&mut self) -> &mut Self {
        if self.cancel_animation() {
            let current = self.get_current();
            if let Some(segment) = self.segments.get_mut(current) {
                segment.reset();
            }
        }
        *self.current.write() = 0;
        self
    }

    /// Gets the total duration of the animation.
    ///
    /// The duration is determined by the sum of segment durations.
    pub fn get_duration(&self) -> u64 {
        self.segments
            .iter()
            .map(|segment| segment.get_duration())
            .sum()
    }

    /// Gets the current play time.
    /// @todo fix: because we clone self on .play(), the progress is no longer available on segment.
    pub fn get_progress(&self) -> u64 {
        let current_segment_index = self.current.read().clone();
        match self.segments.get(current_segment_index) {
            None => 0,
            Some(segment_playing) => segment_playing.get_progress(),
        }
    }

    /// Indicates if the animation is currently playing.
    pub fn is_playing(&self) -> bool {
        self.interval.read().as_ref().is_some()
    }

    /// Inner helper: cancel the animation and return a flag indicating if it was running.
    fn cancel_animation(&mut self) -> bool {
        let was_running = match self.interval.read().as_ref() {
            None => false,
            Some(handler) => {
                handler.abort();
                true
            }
        };
        if was_running {
            *self.interval.write() = None;
        }
        was_running
    }
}

// ########################################
// Conversion helpers

impl From<Segment> for Animation {
    fn from(segment: Segment) -> Self {
        Animation::default().with_segment(segment)
    }
}

impl From<Track> for Animation {
    fn from(track: Track) -> Self {
        Animation::from(Segment::from(track))
    }
}

// ########################################
// Simple getters and setters
impl Animation {
    /// Returns the list of segments in the animation.
    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.segments
    }
    /// Returns the index of the currently running segment.
    pub fn get_current(&self) -> usize {
        self.current.read().clone()
    }

    /// Sets the list of segments for the animation.
    pub fn set_segments(mut self, segments: Vec<Segment>) -> Self {
        self.segments = segments;
        self
    }

    /// Adds a new segment to the animation.
    ///
    /// This segment will be enqueued at the end of current segment list.
    pub fn with_segment(mut self, segment: Segment) -> Self {
        self.segments.push(segment);
        self
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            segments: vec![],
            current: Arc::new(RwLock::new(0)),
            interval: Arc::new(RwLock::new(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use crate::animation::Keyframe;
    use crate::mocks::actuator::MockActuator;
    use crate::pause;

    use super::*;

    fn create_animation() -> Animation {
        let segment = Segment::from(
            Track::new(MockActuator::new(40)).with_keyframe(Keyframe::new(100, 0, 190)),
        );
        Animation::default()
            .with_segment(segment.clone())
            .with_segment(segment.clone())
            .with_segment(segment.clone())
            .with_segment(segment.clone())
            .with_segment(segment.clone())
            .with_segment(segment.clone())
    }

    #[test]
    fn test_animation() {
        let animation = create_animation();

        assert_eq!(animation.get_current(), 0);
        assert_eq!(animation.get_segments().len(), 6);
        assert_eq!(animation.get_duration(), 190 * 6);

        let animation = animation.set_segments(vec![]);
        assert_eq!(animation.get_segments().len(), 0);
        assert_eq!(animation.get_duration(), 0);
    }

    #[test]
    fn test_animation_converters() {
        let animation_from_track = Animation::from(Track::new(MockActuator::new(40)));
        assert_eq!(animation_from_track.get_current(), 0);
        assert_eq!(animation_from_track.get_segments().len(), 1);

        let animation_from_segment = Animation::from(Segment::default());
        assert_eq!(animation_from_segment.get_current(), 0);
        assert_eq!(animation_from_segment.get_segments().len(), 1);
    }

    #[serial]
    #[hermes_macros::test]
    async fn test_play_animation() {
        let mut animation = create_animation();

        assert_eq!(animation.get_current(), 0);
        assert_eq!(animation.get_progress(), 0);
        assert!(!animation.is_playing());
        animation.play();
        assert!(animation.is_playing());
        assert!(animation.is_playing());
        // @todo test "complete" event.
        pause!(220);
        // assert!(animation.get_progress() > 0);
        assert_eq!(animation.get_current(), 1);
        pause!(220);
        assert_eq!(animation.get_current(), 2);
        pause!(220);
        assert_eq!(animation.get_current(), 3);
        pause!(220);
        assert_eq!(animation.get_current(), 4);
        pause!(220);
        assert_eq!(animation.get_current(), 5);
        pause!(220);
        assert_eq!(animation.get_current(), 0);
    }

    #[serial]
    #[hermes_macros::test]
    async fn test_animation_controls() {
        let mut animation = create_animation();
        assert_eq!(animation.get_current(), 0);

        // Animation playing
        animation.play();
        pause!(250);
        assert_eq!(animation.get_current(), 1);

        // Animation paused
        animation.pause();
        pause!(220);
        assert!(animation.interval.read().is_none());
        assert_eq!(animation.get_current(), 1);

        // Animation resumed
        animation.play();
        pause!(250);
        assert_eq!(animation.get_current(), 2);

        // Animation skipped
        animation.next();
        animation.next();
        assert_eq!(animation.get_current(), 4);

        // Animation stopped
        animation.stop();
        assert!(animation.interval.read().is_none());
        assert_eq!(animation.get_current(), 0);
    }

    #[serial]
    #[hermes_macros::test]
    async fn test_animation_skip() {
        // Start and pause the animation during segment 1.
        let mut animation = create_animation();
        assert_eq!(animation.get_current(), 0);
        animation.play();
        pause!(230);
        assert_eq!(animation.get_current(), 1);
        animation.pause();
        pause!(300);
        assert_eq!(animation.get_current(), 1);

        // Paused animation is skipped to next in pause mode.
        animation.next();
        pause!(300);
        assert_eq!(animation.get_current(), 2);

        // Playing animation is skipped to next in playing mode:
        animation.play().next();
        pause!(250);
        assert_eq!(animation.get_current(), 4);

        // Once on the last segment, animation is restarted:
        animation.next().next().next();
        assert_eq!(animation.get_current(), 1);
    }
}
