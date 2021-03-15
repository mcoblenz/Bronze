use crate::turtle::Turtle;

use std::fmt;

// All the turtles live on campus.
pub struct Campus {
    turtles: Vec<Turtle>,
}

impl Campus {
    pub fn new(initial_turtles: u32) -> Campus {
        let mut turtles = Vec::new();
        for _i in 0..initial_turtles {
            turtles.push(Turtle::spawn());
        }

        Campus {turtles}
    }
}

impl fmt::Debug for Campus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in &self.turtles {
            write!(f, "{:?}\n", t)?;
        }
        Ok(())
    }
}