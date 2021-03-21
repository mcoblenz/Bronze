#[derive(Debug)]
pub struct World {
}

impl World {
    pub fn sound(utterance: String) {
        println!("{}", utterance);
    }
}

pub trait TurtlePower {
    fn activate(&mut self, world: &mut World);
}

impl core::fmt::Debug for dyn TurtlePower {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TurtlePower")
    }
}

#[derive(Debug)]
pub struct Wand { 
    charges: u32,
}

impl Wand {
    pub fn new(charges: u32) -> Wand {
        Wand {charges}
    }
}

impl TurtlePower for Wand {
    /**
     * If 'charges' is positive, should play sound "ZAP!" and decrement 'charges.'
     * Otherwise, should do nothing.
     */
    fn activate(&mut self, world: &mut World) {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct Crystal {
}

impl Crystal {
    pub fn new() -> Crystal {
        Crystal{}
    }
}

impl TurtlePower for Crystal {


    /**
     * Crystals last forever. Activate should play a creepy "WOOOOOO" sound.
     */
    fn activate(&mut self, world: &mut World) {
        unimplemented!();
    }
}