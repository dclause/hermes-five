use std::fmt::{Display, Formatter};

use crate::animation::Keyframe;
use crate::devices::Actuator;
use crate::errors::Error;
use crate::utils::Range;
use crate::utils::scale::Scalable;

#[derive(Clone, Debug)]
pub struct Track {
    // @todo keep?
    name: String,

    /// The [`Actuator`] device that this track is associated with.
    /// All keyframes' [`Keyframe::target`] values will reference this device.
    device: Box<dyn Actuator>,

    /// The [`Keyframe`]s belonging to this track.
    keyframes: Vec<Keyframe>,

    /// Inner: keyframe history.
    previous: u16,
    current: u16,
}

impl Track {
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

    /// Inner function: play all keyframes between the given timestamps.
    pub(crate) fn play_frame<F: Into<Range<u64>>>(&mut self, frame: F) -> Result<(), Error> {
        let frame = frame.into();
        // Get the keyframe to be played: the last one that
        let keyframe = self.get_best_keyframe(frame);

        match keyframe {
            None => Ok(()),
            Some(keyframe) => {
                self.update_history(&keyframe);

                // println!(" - Track: {:?}", frame.end - keyframe.get_start());

                let progress = keyframe.get_progress(frame.end);
                let value: f64 = progress.scale(0.0, 1.0, self.previous, keyframe.get_target());
                // println!("      - position: {}", value);
                self.device.set_state(value)
            }
        }
    }

    /// Inner function: returns the most appropriate keyframe corresponding to the given frame.
    fn get_best_keyframe<'a, R: Into<Range<u64>>>(&mut self, frame: R) -> Option<Keyframe> {
        let frame = frame.into();
        // Get the keyframe to be played: the last one that
        self.keyframes
            .iter()
            .filter(|kf| {
                kf.get_start() >= frame.start && kf.get_start() < frame.end ||  // case keyframe starts during the interval
                    kf.get_end() >= frame.start && kf.get_end() < frame.end || // case keyframe ends during the interval
                    kf.get_start() <= frame.start && kf.get_end() > frame.end // case keyframe is running during the interval
            })
            .max_by(|a, b| a.get_end().cmp(&b.get_end()))
            .cloned()
    }

    /// Push the keyframe in history if we changed.
    fn update_history(&mut self, next_keyframe: &Keyframe) {
        if self.current != next_keyframe.get_target() {
            self.previous = self.current;
            self.current = next_keyframe.get_target();
        }
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "keyframe: {}, duration: {}ms",
            self.keyframes.len(),
            self.get_duration()
        )
    }
}

// ########################################
// @todo automate

impl Track {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_device(&self) -> &Box<dyn Actuator> {
        &self.device
    }
    pub fn get_keyframes(&self) -> &Vec<Keyframe> {
        &self.keyframes
    }

    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }
    pub fn set_keyframes(mut self, keyframes: Vec<Keyframe>) -> Self {
        self.keyframes = keyframes;
        self
    }

    pub fn with_keyframe(mut self, keyframe: Keyframe) -> Self {
        self.keyframes.push(keyframe);
        self
    }
}
