use crate::turtle::Turtle;

use std::fmt;
use std::rc::Rc;


// All the turtles live on campus.
pub struct Campus {
    turtles: Vec<Rc<Turtle>>,
}

impl Campus {
    pub fn new(initial_turtles: u32) -> Campus {
        let mut turtles = Vec::new();
        for _i in 0..initial_turtles {
            turtles.push(Rc::new(Turtle::spawn()));
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
            Rc::new(Turtle::breed(t1, t2))
        };

        // self.turtles[t1_index].add_child(new_turtle.clone());
        // self.turtles[t2_index].add_child(new_turtle.clone());

        self.turtles.push(new_turtle);
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