use rand_pcg::Pcg64;
use std::cell::RefCell;
use rand::RngCore;
use rand::SeedableRng;

thread_local!(
    pub static RNG: RefCell<Pcg64> = RefCell::new(Pcg64::seed_from_u64(0));
);

fn rand32() -> u32 {
    RNG.with(|r| (*r.borrow_mut()).next_u32())
}

// Chooses bits at random from each of x and y.
pub fn cross32(x: u32, y: u32) -> u32 {
    let r: u32 = rand32();

    // ! is bitwise not in Rust.
    (x & r) + (y & !r)
}

// Chooses bits at random from each of x and y.
pub fn cross8(x: u8, y: u8) -> u8 {
    let r: u8 = (rand32() & 0xff) as u8;

    // ! is bitwise not in Rust.
    (x & r) + (y & !r)
}

// Basic flavor preferences.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Flavor {
    Sour,
    Sweet,
    Salty,
    Bitter, 
    Umami
}

impl Flavor {
    pub fn random_flavor() -> Flavor {
        let r: u32 = rand32() % 5;
        match r {
            0 => Flavor::Sour,
            1 => Flavor::Sweet,
            2 => Flavor::Salty,
            3 => Flavor::Bitter,
            4 => Flavor::Umami,
            _ => panic!("bug in random_flavor")
        }
    }
}
