#[derive(Copy, Clone)]
pub struct Range {
    pub min: u16,
    pub max: u16,
}

impl From<[u16; 2]> for Range {
    fn from(value: [u16; 2]) -> Self {
        Self {
            min: value[0],
            max: value[1],
        }
    }
}
