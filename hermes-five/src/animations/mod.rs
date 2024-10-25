//! Defines animations to interpolate movements between keyframes of actuator states at a given time.

mod animation;
mod keyframe;
mod segment;
mod track;

pub use animation::{Animation, AnimationEvent};
pub use keyframe::Keyframe;
pub use segment::Segment;
pub use track::Track;
