use crate::utils::Easing;

#[derive(Clone)]
pub struct Keyframe {
    /// The target device state.
    target: u16,
    /// The time (in ms) this keyframe starts.
    start: u64,
    /// The time (in ms) this keyframe ends (auto-calculated).
    end: u64,
    /// The duration (in ms) taken to reach the target.
    duration: u32,
    /// An easing function applied the value while moving toward the target (default: Linear).
    transition: Easing,
}

impl Keyframe {
    pub fn new(target: u16, start: u64, duration: u32) -> Keyframe {
        Keyframe {
            target,
            start,
            duration,
            end: start + duration as u64,
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
    pub fn get_start(&self) -> u64 {
        self.start
    }
    pub fn get_duration(&self) -> u32 {
        self.duration
    }
    pub fn get_end(&self) -> u64 {
        self.end
    }
    pub fn get_transition(&self) -> Easing {
        self.transition
    }

    pub fn set_target(mut self, target: u16) -> Self {
        self.target = target;
        self
    }
    pub fn set_start(mut self, start: u64) -> Self {
        self.start = start;
        self.end = start + self.duration as u64;
        self
    }
    pub fn set_duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self.end = self.start + duration as u64;
        self
    }
    pub fn set_transition(mut self, transition: Easing) -> Self {
        self.transition = transition;
        self
    }
}
