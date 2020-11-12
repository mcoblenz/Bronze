use std::fmt;

enum DevelopmentLevel {
    Rubble,
    Vacant,
    LowDensity,
    HighDensity,
}

pub trait Tile {
    // For now, we render in plaintext only.
    // fn print(&self) -> String;
}

pub struct EmptyTile {}

impl Tile for EmptyTile {
    // fn print(&self) -> String {
    //     String::from(" ")
    // }
}

impl fmt::Display for dyn Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " ")
    }
}


pub struct ZonedTile {
    development_level: DevelopmentLevel,
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