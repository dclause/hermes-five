use std::fmt::{Display, Formatter};

use crate::animation::Keyframe;
use crate::devices::Actuator;
use crate::errors::Error;
use crate::pause;

#[derive(Clone)]
pub struct Track {
    // @todo keep?
    name: String,

    /// The [`Actuator`] device that this track is associated with.
    /// All keyframes' [`Keyframe::target`] values will reference this device.
    device: Box<dyn Actuator>,

    /// The [`Keyframe`]s belonging to this track.
    keyframes: Vec<Keyframe>,
}

impl Track {
    pub fn new<T: Actuator + 'static>(device: T) -> Self {
        Self {
            name: String::from("New track"),
            device: Box::new(device),
            keyframes: vec![],
        }
    }

    pub fn get_duration(&self) -> u64 {
        match self.keyframes.len() > 0 {
            false => 0,
            true => {
                let last_keyframe = self
                    .keyframes
                    .iter()
                    .max_by(|x, y| {
                        (x.get_start() + x.get_duration() as u64)
                            .cmp(&(y.get_start() + y.get_duration() as u64))
                    })
                    .unwrap();
                last_keyframe.get_start() + last_keyframe.get_duration() as u64
            }
        }
    }

    /// Inner function: play all keyframes between the given timestamps.
    pub(crate) fn play_between(&mut self, start: u64, end: u64) -> Result<(), Error> {
        // Get all keyframes to be played before the next frame.
        let keyframes: Vec<Keyframe> = self
            .keyframes
            .iter()
            .filter(|kf| kf.get_start() >= start && kf.get_start() < end)
            .cloned()
            .collect();

        println!(
            " - Track [{}] play between {} - {}: {} keyframes",
            self,
            start,
            end,
            keyframes.len()
        );

        pause!(100);

        // for keyframe in keyframes {
        //     self.device.set_state(keyframe.get_target()).update()
        // }

        // Play the keyframes.
        Ok(())
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
