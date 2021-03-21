use crate::turtle::{Color, Turtle};
use crate::cookbook::{Cookbook};
use crate::genetics::*;

use std::fmt;
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use rand::prelude::*;


// All the turtles live on campus.
pub struct Campus {
    cookbook: Cookbook,
    turtles: Vec<Rc<RefCell<Turtle>>>,
}

impl Campus {
    pub fn new(initial_turtles: u32) -> Campus {
        let cookbook = Cookbook::new();
        let mut rng = rand::thread_rng();

        let mut turtles = Vec::new();
        for _i in 0..initial_turtles {
            let new_turtle = Turtle::new(
                rng.gen::<u32>() % 10, 
                Flavor::random_flavor(),
                Color::new(rng.gen(), rng.gen(), rng.gen()),
            );

            turtles.push(Rc::new(RefCell::new(new_turtle)));
        }

        Campus {cookbook, turtles}
    }

    pub fn size(&self) -> usize {
        self.turtles.len()
    }

    pub fn breed_turtles(&mut self, t1_index: usize, t2_index: usize) {
        // We need to make sure t1 and t2 go out of scope before the last line, because t1 borrows turtles immutably. When t1 gets dropped, it might re-use the borrow, so that needs to happen BEFORE turtles gets borrowed mutably.

        let new_turtle = {
            let t1 = self.turtles[t1_index].clone();
            let t2 = self.turtles[t2_index].clone();
            // let mut t1 = (*self.turtles[t1_index]).borrow_mut();
            // let mut t2 = (*self.turtles[t2_index]).borrow_mut();

            Rc::new(RefCell::new(Turtle::breed(t1, t2)))
        };

        self.turtles[t1_index].borrow_mut().add_child(new_turtle.clone());
        self.turtles[t2_index].borrow_mut().add_child(new_turtle.clone());

        self.turtles.push(new_turtle);
    }

    pub fn turtles(&self) -> std::slice::Iter<Rc<RefCell<Turtle>>> {
        self.turtles.iter()
    }

    pub fn fastest_walker(&self) -> Option<Ref<Turtle>> {
        let mut fastest = None;

        for turtle in self.turtles() {
            match fastest {
                None => fastest = Some(turtle),
                Some(t) => 
                    if turtle.borrow().walking_speed() > t.borrow().walking_speed() {
                        fastest = Some(turtle);
                    }
            }
        }

        fastest.map(|f| f.borrow())
    }

    pub fn school(&self) {
        for turtle in self.turtles() {
            turtle.borrow_mut().teach_children();
        }
    }

    pub fn paint_turtle(&self, turtle_index: usize, new_color: Color) {
        self.turtles[turtle_index].borrow_mut().set_color(new_color);
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
