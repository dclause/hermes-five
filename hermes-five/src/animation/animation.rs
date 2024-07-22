use std::sync::Arc;

use crate::animation::{Segment, Track};
use crate::errors::Error;
use crate::pause;
use crate::utils::task;
use crate::utils::task::TaskHandler;

#[derive(Clone, Debug)]
pub struct Animation {
    // @todo keep?
    // name: String,
    /// The ordered list of [`AnimationSegment`].
    segments: Vec<Segment>,
    /// The current running Segment.
    current: usize,

    /// Determines whether the segment should replay in a loop (starting from the [`Segment::loopback`] time).
    repeat: bool,

    /// Inner handler to the task running the animation.
    interval: Arc<Option<TaskHandler>>,
}

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

impl Animation {
    /// Play the animation.
    pub async fn play(&mut self) -> &Self {
        let mut self_clone = self.clone();
        self.interval = Arc::new(Some(
            task::run(async move {
                match self_clone.is_repeat() {
                    true => loop {
                        self_clone.play_once()?;
                        pause!(1);
                    },
                    false => self_clone.play_once()?,
                }
                Ok(())
            })
            .unwrap(),
        ));

        self
    }

    /// Pauses the animation.
    pub fn pause(&self) -> &Self {
        match &self.interval.as_ref() {
            None => {}
            Some(handler) => handler.abort(),
        }
        self
    }

    /// Stops the animation and reset it.
    pub fn stop(&mut self) -> &Self {
        match &self.interval.as_ref() {
            None => {}
            Some(handler) => {
                self.segments.get_mut(self.current).unwrap().reset();
                self.current = 0;
                handler.abort();
            }
        }
        self
    }

    /// Inner function: play all segment once.
    fn play_once(&mut self) -> Result<(), Error> {
        let starting_segment = self.current;
        for current in starting_segment..self.segments.len() {
            self.current = current;

            let segment_playing = self.segments.get_mut(self.current).unwrap();
            segment_playing.play()?;
        }
        self.current = 0; // reset
        Ok(())
    }
}

// ########################################
// @todo automate

impl Animation {
    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.segments
    }
    pub fn get_current(&self) -> usize {
        self.current
    }
    pub fn is_repeat(&self) -> bool {
        self.repeat
    }

    pub fn set_segments(mut self, segments: Vec<Segment>) -> Self {
        self.segments = segments;
        self
    }
    pub fn set_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_segment(mut self, segment: Segment) -> Self {
        self.segments.push(segment);
        self
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            segments: vec![],
            current: 0,
            repeat: false,
            interval: Arc::new(None),
        }
    }
}
