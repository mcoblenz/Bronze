use std::fmt;

pub enum DevelopmentLevel {
    Rubble,
    Vacant,
    LowDensity,
    HighDensity,
}

pub trait Tile: fmt::Display {
    // For now, we render in plaintext only.
}

impl bronze::GcTrace for dyn Tile {}

pub struct EmptyTile {}

impl Tile for EmptyTile {
}

impl fmt::Display for EmptyTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " ")
    }
}

impl bronze::GcTrace for EmptyTile {}


pub struct ZonedTile {
    pub development_level: DevelopmentLevel,
}

impl ZonedTile {
    pub fn new() -> Self {
        ZonedTile {development_level: DevelopmentLevel::Vacant}
    }
}

impl Tile for ZonedTile {
    
}

impl fmt::Display for ZonedTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match &self.development_level {
            DevelopmentLevel::Rubble => "R",
            DevelopmentLevel::Vacant => "V",
            DevelopmentLevel::LowDensity => "L",
            DevelopmentLevel::HighDensity => "H"
        };
        write!(f, "{}", str)
    }
}

impl bronze::GcTrace for ZonedTile {}
