//! Create animations.
//!
//! This module allows the creation of [`Animation`] made of [`Keyframe`] (state of registered devices) arranged as [`Segment`].
//!
//! - An [`Animation`] contains one or more ordered [`Segment`].
//! - A [`Segment`] contains one or more [`Track`]: one per [`Output`](crate::devices::Output) device to animate.
//! - A [`Track`] contains as many [`Keyframe`] as required: they are states that the device must have at a given time.
//!   The path (succession of intermediate state) taken by the device in between the keyframes is automatically interpolated following an [`Easing`] transition.

mod animation;
mod easing;
mod keyframe;
mod segment;
mod track;

pub use animation::{Animation, AnimationEvent};
pub use easing::Easing;
pub use keyframe::Keyframe;
pub use segment::Segment;
pub use track::Track;
