use crate::animation::Keyframe;
use crate::devices::Actuator;

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
