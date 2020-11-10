// use rand::Rng;
use std::fmt;

enum DevelopmentLevel {
    Rubble,
    Vacant,
    LowDensity,
    HighDensity,
}

pub trait Tile {
    // For now, we render in plaintext only.
    fn print(&self) -> String;
    
    // Cause the tile to take one step forward in the simulation.
    fn step(&self); 
}

pub struct EmptyTile {}

impl Tile for EmptyTile {
    fn print(&self) -> String {
        String::from(" ")
    }

    fn step(&self) {
        // Nothing to do.
    }
}

impl fmt::Display for dyn Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " ")
    }
}


struct ZonedTile {
    development_level: DevelopmentLevel,
}

impl ZonedTile {
    fn step() {
        // let mut rng = rand::thread_rng();

        // let n = rng.gen();
    }
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