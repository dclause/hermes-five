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
/// let board = Board::run();
/// board.on("ready", |board: Board| async move {
///     let servo = Servo::new(&board, 9, 0).unwrap();
///
///     let mut animation = Animation::from(
///         Segment::from(
///             Track::new(servo)
///                 .with_keyframe(Keyframe::new(180, 0, 500).set_transition(Easing::SineInOut))
///                 .with_keyframe(Keyframe::new(90, 1000, 2000).set_transition(Easing::SineInOut)),
///         )
///         .set_fps(100)
///         .set_repeat(true),
///     );
///
///     animation.play().await;
///     pause!(3000);
///
///     animation.stop();
/// }).await;
/// ```
///
/// # Fields
///
/// - `name`: The name of the animation.
/// - `segments`: The ordered list of animation [`Segment`]s.
/// - `current`: The index of the currently running [`Segment`] (starting at 0).
#[derive(Clone, Debug)]
pub struct Animation {
    /// The animation name.
    name: String,
    /// The ordered list of animation [`Segment`].
    segments: Vec<Segment>,
    /// The index of current running [`Segment`].
    current: Arc<RwLock<usize>>,

    /// Inner handler to the task running the animation.
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
    /// Skipping the current segment does not pause / resume the animation: if it was running, it
    /// continues to do so (from the beginning of next segment).
    pub fn next(&mut self) -> &mut Self {
        // Stop the current animation (do not reuse `.stop()` to avoid triggering the stop event).
        let was_running = self.cancel_animation();

        // Move to the next segment if we are not at the end.
        let current = self.get_current();
        if current < self.segments.len() - 1 {
            *self.current.write() = current + 1;
            // Restart the animation from the beginning of the next segment, if it was running.
            if was_running {
                self.play();
            }
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
    /// Returns the name of the segment.
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    /// Returns the list of segments in the animation.
    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.segments
    }
    /// Returns the index of the currently running segment.
    pub fn get_current(&self) -> usize {
        self.current.read().clone()
    }

    /// Sets the name of the segment and returns the updated segment.
    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
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
            name: String::from("New animation"),
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
    use crate::pause;
    use crate::tests::mocks::actuator::MockActuator;

    use super::*;

    fn create_animation() -> Animation {
        let segment = Segment::from(
            Track::new(MockActuator::new(40)).with_keyframe(Keyframe::new(100, 0, 200)),
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

        assert_eq!(animation.get_name(), "New animation");
        assert_eq!(animation.get_current(), 0);
        assert_eq!(animation.get_segments().len(), 6);

        let animation = animation.set_name("Test animation").set_segments(vec![]);
        assert_eq!(animation.get_name(), "Test animation");
        assert_eq!(animation.get_segments().len(), 0);
    }

    #[test]
    fn test_animation_converters() {
        let animation_from_track = Animation::from(Track::new(MockActuator::new(40)));
        assert_eq!(animation_from_track.get_name(), "New animation");
        assert_eq!(animation_from_track.get_current(), 0);
        assert_eq!(animation_from_track.get_segments().len(), 1);

        let animation_from_segment = Animation::from(Segment::default());
        assert_eq!(animation_from_segment.get_name(), "New animation");
        assert_eq!(animation_from_segment.get_current(), 0);
        assert_eq!(animation_from_segment.get_segments().len(), 1);
    }

    #[serial]
    #[hermes_macros::test]
    async fn test_play_animation() {
        let mut animation = create_animation();

        assert_eq!(animation.get_current(), 0);
        animation.play();
        // @todo test "complete" event.
        pause!(220);
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
        pause!(220);
        animation.pause();
        assert_eq!(animation.get_current(), 1);

        // Paused animation is skipped to next in pause mode.
        animation.next();
        pause!(220);
        assert_eq!(animation.get_current(), 2);

        // Playing animation is skipped to next in playing mode:
        animation.play().next();
        pause!(220);
        assert_eq!(animation.get_current(), 4);

        // Once on the last segment, animation is stopped when skipped:
        animation.next().next().next();
        assert!(animation.interval.read().is_none());
        assert_eq!(animation.get_current(), 5);
    }
}
