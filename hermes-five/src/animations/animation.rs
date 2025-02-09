use std::fmt::{Display, Formatter};

use parking_lot::RwLock;

use crate::errors::Error;
use crate::utils::{task, EventHandler, EventManager, TaskHandler};

use crate::animations::{Segment, Track};
use std::sync::Arc;

/// Lists all events a Animation can emit/listen.
pub enum AnimationEvent {
    /// Triggered when the animation starts.
    OnSegmentDone,
    /// Triggered when the animation starts.
    OnStart,
    /// Triggered when the animation finishes.
    OnComplete,
}

/// Convert events to string to facilitate usage with [`EventManager`].
impl From<AnimationEvent> for String {
    fn from(event: AnimationEvent) -> Self {
        let event = match event {
            AnimationEvent::OnSegmentDone => "segment_done",
            AnimationEvent::OnStart => "start",
            AnimationEvent::OnComplete => "complete",
        };
        event.into()
    }
}

/// Represents an animation: a collection of ordered [`Segment`] to be run in sequence.
///
/// - An animation can be played, paused, resumed, and stopped.
/// - Each [`Segment`] in the animation is played in order (eventually looped, see [`Segment`]) until all are done.
///
/// # Example
///
/// Here is an example of an animation made of a single segment that sweeps a servo indefinitely,
/// from position 0° to 180° and back:
/// (sidenote: prefer use [`Servo::sweep()`](crate::devices::Servo::sweep()) helper for this purpose).
/// ```
/// use hermes_five::pause;
/// use hermes_five::animations::{Animation, Easing, Keyframe, Segment, Track};
/// use hermes_five::hardware::{Board, BoardEvent};
/// use hermes_five::devices::Servo;
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
    /// The event manager for the animation.
    #[cfg_attr(feature = "serde", serde(skip))]
    events: EventManager,
}

// ########################################

impl Default for Animation {
    fn default() -> Self {
        Self {
            segments: vec![],
            current: Arc::new(RwLock::new(0)),
            interval: Arc::new(RwLock::new(None)),
            events: Default::default(),
        }
    }
}

impl Animation {
    /// Starts or resumes the animation.
    ///
    /// The animation will start from the current segment or from the beginning if it was stopped.
    pub fn play(&mut self) -> &mut Self {
        let events_clone = self.events.clone();
        let mut self_clone = self.clone();

        self.events.emit(AnimationEvent::OnStart, self.clone());
        if self.get_duration() > 0 {
            let handler = task::run(async move {
                // Loop through the segments and run them one by one.
                for index in self_clone.get_current()..self_clone.segments.len() {
                    *self_clone.current.write() = index;

                    // Retrieve the currently running segment.
                    let segment_playing = self_clone.segments.get_mut(index).unwrap();
                    segment_playing.play().await?;
                    events_clone.emit(AnimationEvent::OnSegmentDone, segment_playing.clone());
                }

                *self_clone.current.write() = 0; // reset to the beginning
                *self_clone.interval.write() = None;
                events_clone.emit(AnimationEvent::OnComplete, self_clone);
                Ok(())
            })
            .unwrap();
            *self.interval.write() = Some(handler);
        }

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
    ///   continues to do so (from the beginning of next segment).
    /// - If already on the last segment, the animation loop to the first one, hence, restart.
    #[allow(clippy::should_implement_trait)]
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

    /// Indicates if the animation is currently playing.
    pub fn is_playing(&self) -> bool {
        self.interval.read().as_ref().is_some()
    }

    /// Gets the total duration of the animation.
    ///
    /// The duration is determined by the sum of segment durations or u64::MAX if a segment is on repeat mode.
    pub fn get_duration(&self) -> u64 {
        self.segments
            .iter()
            .map(|segment| match segment.is_repeat() {
                true => u64::MAX,
                false => segment.get_duration(),
            })
            .sum()
    }

    /// Gets the current play time.
    /// @todo fix: because we clone self on .play(), the progress is no longer available on segment.
    pub fn get_progress(&self) -> u64 {
        let current_segment_index = *self.current.read();
        match self.segments.get(current_segment_index) {
            None => 0,
            Some(segment_playing) => segment_playing.get_progress(),
        }
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

    /// Returns the list of segments in the animation.
    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.segments
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

    /// Returns the index of the currently running segment.
    pub fn get_current(&self) -> usize {
        *self.current.read()
    }

    /// Sets the index of the currently running segment.
    pub fn set_current(&self, index: usize) {
        *self.current.write() = index;
    }

    // ########################################
    // Event related functions

    /// Registers a callback to be executed on a given event.
    ///
    /// Available events for an animation are defined by the enum: [`AnimationEvent`]:
    /// - **`OnSegmentDone` | `segment_done`**: Triggered when a segment is done.    
    ///    _The callback must receive the following parameter: `|_: Segment| { ... }`_
    /// - **`OnStart` | `start`**: Triggered when the animation starts.    
    ///    _The callback must receive the following parameter: `|_: Animation| { ... }`_
    /// - **`OnComplete` | `complete`**: Triggered when the animation ends.    
    ///    _The callback must receive the following parameter: `|_: Animation| { ... }`_
    ///
    /// # Example
    /// ```
    /// use hermes_five::hardware::Board;
    /// use hermes_five::hardware::BoardEvent;
    /// use hermes_five::devices::{Output, Led};
    /// use hermes_five::animations::{Animation, AnimationEvent, Easing};
    ///
    /// #[hermes_five::runtime]
    /// async fn main() {
    ///     let board = Board::run();
    ///     board.on(BoardEvent::OnReady, |board: Board| async move {
    ///         let mut led = Led::new(&board, 11, false)?;
    ///         // This is a dummy animation (does nothing).
    ///         let animation = Animation::default();
    ///
    ///         animation.on(AnimationEvent::OnStart, |_: Animation| async move {
    ///             println!("Animation has started");
    ///             Ok(())
    ///         });
    ///         animation.on(AnimationEvent::OnComplete, |_: Animation| async move {
    ///             println!("Animation done");
    ///             Ok(())
    ///         });
    ///         Ok(())
    ///     });
    /// }
    /// ```
    pub fn on<S, F, T, Fut>(&self, event: S, callback: F) -> EventHandler
    where
        S: Into<String>,
        T: 'static + Send + Sync + Clone,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.events.on(event, callback)
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

impl Display for Animation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Animation [duration={}, segments={}]",
            match self.get_duration() {
                u64::MAX => String::from("INF"),
                duration => format!("{}ms", duration),
            },
            self.segments.len()
        )?;
        for segment in &self.segments {
            writeln!(
                f,
                "  Segment [duration={}ms, repeat={}, fps={}, speed={}] :",
                segment.get_duration(),
                segment.is_repeat(),
                segment.get_fps(),
                segment.get_speed()
            )?;
            for track in segment.get_tracks() {
                writeln!(
                    f,
                    "   Track [duration={}ms, device={}]:",
                    track.get_duration(),
                    track.get_device()
                )?;
                for keyframe in track.get_keyframes() {
                    writeln!(
                        f,
                        "      Keyframe {}ms to {}ms: {} [transition={:?}]",
                        keyframe.get_start(),
                        keyframe.get_end(),
                        keyframe.get_target(),
                        keyframe.get_transition(),
                    )?;
                }
            }
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use crate::animations::Keyframe;
    use crate::mocks::output_device::MockOutputDevice;
    use crate::pause;

    use super::*;

    fn create_animation() -> Animation {
        let segment = Segment::from(
            Track::new(MockOutputDevice::new(40)).with_keyframe(Keyframe::new(100, 0, 190)),
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
        let animation = Animation::default();
        assert_eq!(animation.get_current(), 0);
        assert_eq!(animation.get_progress(), 0);
        assert!(!animation.is_playing());

        let animation = create_animation();

        assert_eq!(animation.get_current(), 0);
        assert_eq!(animation.get_segments().len(), 6);
        assert_eq!(animation.get_duration(), 190 * 6);

        let animation = animation.set_segments(vec![]);
        assert_eq!(animation.get_segments().len(), 0);
        assert_eq!(animation.get_duration(), 0);

        let animation = animation.with_segment(
            Segment::from(
                Track::new(MockOutputDevice::new(40)).with_keyframe(Keyframe::new(100, 0, 190)),
            )
            .set_repeat(true),
        );
        assert_eq!(animation.get_segments().len(), 1);
        assert_eq!(animation.get_duration(), u64::MAX);
        assert_eq!(animation.to_string(), "Animation [duration=INF, segments=1]\n  Segment [duration=190ms, repeat=true, fps=60, speed=100] :\n   Track [duration=190ms, device=MockActuator [state=40]]:\n      Keyframe 0ms to 190ms: 100 [transition=Linear]\n");

        let animation = animation.set_segments(vec![Segment::from(
            Track::new(MockOutputDevice::new(40)).with_keyframe(Keyframe::new(100, 0, 190)),
        )]);
        assert_eq!(animation.get_duration(), 190);
        assert_eq!(animation.to_string(), "Animation [duration=190ms, segments=1]\n  Segment [duration=190ms, repeat=false, fps=60, speed=100] :\n   Track [duration=190ms, device=MockActuator [state=40]]:\n      Keyframe 0ms to 190ms: 100 [transition=Linear]\n");
    }

    #[test]
    fn test_animation_converters() {
        let animation_from_track = Animation::from(Track::new(MockOutputDevice::new(40)));
        assert_eq!(animation_from_track.get_current(), 0);
        assert_eq!(animation_from_track.get_segments().len(), 1);

        let animation_from_segment = Animation::from(Segment::default());
        assert_eq!(animation_from_segment.get_current(), 0);
        assert_eq!(animation_from_segment.get_segments().len(), 1);
    }

    #[serial]
    #[hermes_five_macros::test]
    async fn test_play_animation() {
        let mut animation = create_animation();
        assert_eq!(animation.get_current(), 0);
        assert_eq!(animation.get_progress(), 0);
        assert!(!animation.is_playing());

        let flag = Arc::new(AtomicBool::new(false));
        let active_segment = Arc::new(AtomicUsize::new(0));

        let moved_flag = flag.clone();
        animation.on(AnimationEvent::OnStart, move |animation: Animation| {
            let captured_flag = moved_flag.clone();
            async move {
                captured_flag.store(true, Ordering::SeqCst);
                assert_eq!(animation.get_current(), 0);
                Ok(())
            }
        });

        let moved_active_segment = active_segment.clone();
        animation.on(AnimationEvent::OnSegmentDone, move |_: Segment| {
            let captured_active_segment = moved_active_segment.clone();
            async move {
                let index = captured_active_segment.load(Ordering::SeqCst) + 1;
                captured_active_segment.store(index, Ordering::SeqCst);
                Ok(())
            }
        });

        let moved_flag = flag.clone();
        animation.on(AnimationEvent::OnComplete, move |animation: Animation| {
            let captured_flag = moved_flag.clone();
            async move {
                captured_flag.store(false, Ordering::SeqCst);
                assert_eq!(animation.get_current(), 5);
                Ok(())
            }
        });

        // Test animation play & event start.
        assert!(!flag.load(Ordering::SeqCst));
        animation.play();
        assert!(animation.is_playing());
        pause!(150);
        // assert_ne!(animation.get_progress(), 0); // @todo fix
        assert!(flag.load(Ordering::SeqCst));

        // Test animation finished & event stop.
        pause!(500);
        assert!(animation.get_current() > 0);
        assert_eq!(
            animation.get_current(),
            active_segment.load(Ordering::SeqCst)
        );
        pause!(1000);
        assert!(!flag.load(Ordering::SeqCst));
    }

    #[serial]
    #[hermes_five_macros::test]
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

        // Animation jump to given segment
        animation.set_current(5);
        assert_eq!(animation.get_current(), 5);

        // Animation stopped
        animation.stop();
        assert!(animation.interval.read().is_none());
        assert_eq!(animation.get_current(), 0);
    }

    #[serial]
    #[hermes_five_macros::test]
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
