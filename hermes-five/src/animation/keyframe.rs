use crate::utils::Easing;

#[derive(Clone)]
pub struct Keyframe {
    /// The target device state.
    target: u16,
    /// The duration (in ms) taken to reach the target.
    duration: u32,
    /// An easing function applied the value while moving toward the target (default: Linear).
    transition: Easing,
}

impl Keyframe {
    pub fn new(target: u16, duration: u32) -> Keyframe {
        Keyframe {
            target,
            duration,
            transition: Easing::default(),
        }
    }
}

// ########################################
// @todo automate

impl Keyframe {
    pub fn get_target(&self) -> u16 {
        self.target
    }
    pub fn get_duration(&self) -> u32 {
        self.duration
    }
    pub fn get_transition(&self) -> Easing {
        self.transition
    }

    pub fn set_target(mut self, target: u16) -> Self {
        self.target = target;
        self
    }
    pub fn set_duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }
    pub fn set_transition(mut self, transition: Easing) -> Self {
        self.transition = transition;
        self
    }
}
