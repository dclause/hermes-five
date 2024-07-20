use crate::animation::Track;
use crate::errors::Error;

#[derive(Clone)]
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

    /// The tracks for this segment.
    tracks: Vec<Track>,
}

impl From<Track> for Segment {
    fn from(track: Track) -> Self {
        Self::default().with_track(track)
    }
}

impl Segment {
    pub(crate) fn play(&self) -> Result<(), Error> {
        println!("start playing segment");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(())
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
            tracks: vec![],
        }
    }
}
