use crate::turtle::Turtle;

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;


// All the turtles live on campus.
pub struct Campus {
    turtles: Vec<Rc<RefCell<Turtle>>>,
}

impl Campus {
    pub fn new(initial_turtles: u32) -> Campus {
        let mut turtles = Vec::new();
        for _i in 0..initial_turtles {
            turtles.push(Rc::new(RefCell::new(Turtle::spawn())));
        }

        Campus {turtles}
    }

    pub fn size(&self) -> usize {
        self.turtles.len()
    }

    pub fn breed_turtles(&mut self, t1_index: usize, t2_index: usize) {
        // We need to make sure t1 and t2 go out of scope before the last line, because t1 borrows turtles immutably. When t1 gets dropped, it might re-use the borrow, so that needs to happen BEFORE turtles gets borrowed mutably.
        let new_turtle = {
            let mut t1 = (*self.turtles[t1_index]).borrow_mut();
            let mut t2 = (*self.turtles[t2_index]).borrow_mut();

            let new_turtle = Rc::new(RefCell::new(Turtle::breed(&t1, &t2)));

            t1.add_child(new_turtle.clone());
            t2.add_child(new_turtle.clone());

            new_turtle
        };

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