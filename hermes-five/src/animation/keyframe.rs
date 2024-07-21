use crate::utils::Easing;
use crate::utils::scale::Scalable;

#[derive(Clone, Debug)]
pub struct Keyframe {
    /// The target device state.
    target: u16,
    /// The time (in ms) this keyframe starts.
    start: u64,
    /// The time (in ms) this keyframe ends (auto-calculated).
    end: u64,
    /// An easing function applied the value while moving toward the target (default: Linear).
    transition: Easing,
}

impl Keyframe {
    pub fn new(target: u16, start: u64, end: u64) -> Keyframe {
        Keyframe {
            target,
            start,
            end,
            transition: Easing::default(),
        }
    }

    pub fn get_duration(&self) -> u64 {
        self.end - self.start
    }

    pub(crate) fn get_progress(&self, time: u64) -> f32 {
        let time = time.clamp(self.start, self.end) as f32;
        let progress = time.scale(self.start as f32, self.end as f32, 0.0, 1.0);
        let progress = self.transition.call(progress);
        // println!("      - progress: {}%", progress * 100f32);
        progress
    }
}

impl PartialEq<Keyframe> for Keyframe {
    fn eq(&self, other: &Keyframe) -> bool {
        self.start != other.start && self.end != other.end
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
    pub fn get_end(&self) -> u64 {
        self.end
    }
    pub fn get_transition(&self) -> Easing {
        self.transition
    }

    pub fn set_transition(mut self, transition: Easing) -> Self {
        self.transition = transition;
        self
    }
}
