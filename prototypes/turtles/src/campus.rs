use crate::turtle::Turtle;

use std::fmt;

// All the turtles live on campus.
pub struct Campus<'a> {
    turtles: Vec<Turtle<'a>>,
}

impl<'a> Campus<'a> {
    pub fn new(initial_turtles: u32) -> Campus<'a> {
        let mut turtles = Vec::new();
        for _i in 0..initial_turtles {
            turtles.push(Turtle::spawn());
        }

        Campus {turtles}
    }

    pub fn size(&self) -> usize {
        self.turtles.len()
    }

    pub fn breed_turtles(&mut self, t1_index: usize, t2_index: usize) {
        let new_turtle = 
        {
            // let slice = self.turtles.as_slice();
            let t1 = &self.turtles[t1_index];
            let t2 = &self.turtles[t2_index];
            Turtle::breed(t1, t2)
        };

        
        // self.turtles[t1_index].add_child(&new_turtle);
        // self.turtles[t2_index].add_child(&new_turtle);

        self.turtles.push(new_turtle);
    }
}

impl<'a> fmt::Debug for Campus<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for t in &self.turtles {
            write!(f, "{:?}\n", t)?;
        }
        Ok(())
    }
}