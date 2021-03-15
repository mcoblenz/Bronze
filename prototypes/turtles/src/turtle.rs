use rand::prelude::*;

// Chooses bits at random from each of x and y.
pub fn cross32(x: u32, y: u32) -> u32 {
    let mut rng = rand::thread_rng();
    let r: u32 = rng.gen();

    (x & r) + (y & !r)
}

// Chooses bits at random from each of x and y.
pub fn cross8(x: u8, y: u8) -> u8 {
    let mut rng = rand::thread_rng();
    let r: u8 = rng.gen();

    (x & r) + (y & !r)
}

#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8
}

impl Color {
    pub fn cross(c1: &Color, c2: &Color) -> Color {
        Color {
            r: cross8(c1.r, c2.r),
            g: cross8(c1.g, c2.g),
            b: cross8(c1.b, c2.b),
        }
    }
}

#[derive(Debug)]
pub struct Turtle {
    walking_speed: u32,
    swimming_speed: u32,
    color: Color,
}


impl Turtle {
    // You normally don't create a new Turtle from nothing; instead, breed
    // two Turtles.
    pub fn breed(p1: &Turtle, p2: &Turtle) -> Turtle {
        Turtle {
            walking_speed: cross32(p1.walking_speed, p2.walking_speed),
            swimming_speed: cross32(p1.swimming_speed, p2.swimming_speed),
            color: Color::cross(&p1.color, &p2.color)
        }
    }

    // This is for use when creating the initial world only.
    pub fn spawn() -> Turtle {
        let mut rng = rand::thread_rng();

        Turtle {
            walking_speed: rng.gen(),
            swimming_speed: rng.gen(),
            color: Color{r: rng.gen(), g: rng.gen(), b: rng.gen()}
        }
    }
}
