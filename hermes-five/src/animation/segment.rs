use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use crate::animation::Track;
use crate::errors::Error;
use crate::pause;

#[derive(Clone, Debug)]
pub struct Segment {
    // @todo keep?
    name: String,

    /// Determines whether the segment should replay in a loop (starting from the [`Segment::loopback`] time).
    repeat: bool,

    /// The point in time (in ms) the animation will restart the loop when `loop` is set to true (default: 0).
    loopback: u32,

    /// Controls the speed of the animation. (default: 100)
    /// Defined as a percentage of standard time. For example:
    /// - 50% means time moves twice as slow, so 1000ms lasts 2000ms in real time.
    /// - 200% means time moves twice as fast, so 1000ms lasts 500ms in real time
    speed: u8,
    /// The number of frames per second (fps) for running the animation (default: 40fps).
    /// - Higher fps results in smoother animations.
    /// - Desired `fps` is not guaranteed to be reached (specially high fps values).
    /// - The `fps` can be overridden for each [`Segment`] in the animation.
    fps: u8,
    /// The tracks for this segment.
    tracks: Vec<Track>,

    /// The current time (in ms) the segment is currently at.
    current_time: u64,
}

impl From<Track> for Segment {
    fn from(track: Track) -> Self {
        Self::default().with_track(track)
    }
}

impl Segment {
    /// Inner function: play the segment.
    pub async fn play(&mut self) -> Result<(), Error> {
        match self.is_repeat() {
            true => loop {
                let start = SystemTime::now();
                self.play_once().await?;
                let end = SystemTime::now();
                let elapsed = end.duration_since(start).unwrap().as_millis();
                println!("played once in {}", elapsed);
                self.current_time = 0;
            },
            false => self.play_once().await?,
        };
        self.current_time = 0;
        Ok(())
    }

    /// Inner function: play all tracks once.
    pub async fn play_once(&mut self) -> Result<(), Error> {
        // println!("Play segment: [{}] at {} fps", self, self.fps);

        let total_duration = self.get_duration();
        // The theoretical time a frame should take.
        let theoretical_frame_duration = 1000u64 / self.fps as u64;
        // The realtime a frame took.
        let mut realtime_frame_duration = 0u64;

        while self.current_time < total_duration {
            let realtime_start = SystemTime::now();

            // The next frame time is
            let next_frame_time =
                self.current_time + theoretical_frame_duration.max(realtime_frame_duration);

            for track in &mut self.tracks {
                track.play_frame([self.current_time, next_frame_time])?;
            }

            let realtime_end = SystemTime::now();
            realtime_frame_duration = realtime_end
                .duration_since(realtime_start)
                .unwrap()
                .as_millis() as u64;

            let remaining_track_time = total_duration - self.current_time;
            let remaining_frame_time = theoretical_frame_duration - realtime_frame_duration;
            // println!(
            //     "=> current time is: {} - pause for {} until {}",
            //     self.current_time,
            //     remaining_frame_time.min(remaining_track_time),
            //     next_frame_time
            // );
            pause!(remaining_frame_time.min(remaining_track_time));
            self.current_time = next_frame_time;
        }

        Ok(())
    }

    pub(crate) fn reset(&mut self) {
        self.current_time = 0;
    }

    pub fn get_duration(&self) -> u64 {
        match self.tracks.len() > 0 {
            false => 0,
            true => {
                let longest_track = self
                    .tracks
                    .iter()
                    .max_by(|x, y| x.get_duration().cmp(&y.get_duration()))
                    .unwrap();
                longest_track.get_duration()
            }
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "tracks: {}, duration: {}ms",
            self.tracks.len(),
            self.get_duration()
        )
    }
}

// ########################################
// @todo automate

impl Segment {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn is_repeat(&self) -> bool {
        self.repeat
    }
    pub fn get_loopback(&self) -> u32 {
        self.loopback
    }
    pub fn get_speed(&self) -> u8 {
        self.speed
    }
    pub fn get_fps(&self) -> u8 {
        self.fps
    }
    pub fn get_tracks(&self) -> Vec<Track> {
        self.tracks.clone()
    }

    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }
    pub fn set_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
    pub fn set_loopback(mut self, loopback: u32) -> Self {
        self.loopback = loopback;
        self
    }
    pub fn set_speed(mut self, speed: u8) -> Self {
        self.speed = speed;
        self
    }
    pub fn set_fps(mut self, fps: u8) -> Self {
        self.fps = fps;
        self
    }
    pub fn set_tracks(mut self, tracks: Vec<Track>) -> Self {
        self.tracks = tracks;
        self
    }

    pub fn with_track(mut self, track: Track) -> Self {
        self.tracks.push(track);
        self
    }
}

impl Default for Segment {
    fn default() -> Self {
        Segment {
            name: String::from("new segment"),
            repeat: false,
            loopback: 0,
            speed: 100,
            fps: 100,
            tracks: vec![],
            current_time: 0,
        }
    }
}
