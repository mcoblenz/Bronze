use std::fmt;
use bronze_derive::*;
use bronze::*;

#[derive(Trace, Finalize)]
pub enum DevelopmentLevel {
    Rubble,
    Vacant,
    LowDensity,
    HighDensity,
}

pub trait Tile: fmt::Display {
    // For now, we render in plaintext only.
}

impl Finalize for dyn Tile {}
unsafe impl GcTrace for dyn Tile {
    custom_trace!(this, {});
}

#[derive(Trace, Finalize)]
pub struct EmptyTile {}

impl Tile for EmptyTile {
}

impl fmt::Display for EmptyTile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " ")
    }
}


#[derive(Trace, Finalize)]
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
