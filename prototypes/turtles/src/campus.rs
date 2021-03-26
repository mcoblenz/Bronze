use crate::turtle::{Color, Turtle};
use crate::turtle_collection::TurtleCollection;
use crate::genetics::*;

use std::fmt;
use std::rc::Rc;
use std::cell::{RefCell, Ref};
use std::collections::HashMap;
use rand::prelude::*;

type TurtleRef = Rc<RefCell<Turtle>>;

// All the turtles live on campus.
pub struct Campus {
    turtles: Vec<TurtleRef>,

    turtle_name_cache: RefCell<HashMap<String, Rc<Vec<TurtleRef>>>>,
}

impl Campus {
    pub fn new(initial_turtles: u32) -> Campus {
        let mut rng = rand::thread_rng();

        let mut turtles = Vec::new();
        for _i in 0..initial_turtles {
            let new_turtle = Turtle::new(
                rng.gen::<u8>().to_string(), // random name
                rng.gen::<u32>() % 10, 
                Flavor::random_flavor(),
                Color::new(rng.gen(), rng.gen(), rng.gen()),
            );

            turtles.push(Rc::new(RefCell::new(new_turtle)));
        }

        let turtle_name_cache = RefCell::new(HashMap::new());
        Campus {turtles, turtle_name_cache}
    }

    pub fn size(&self) -> usize {
        self.turtles.len()
    }

    pub fn breed_turtles(&mut self, t1_index: usize, t2_index: usize, name: String) {
        // We need to make sure t1 and t2 go out of scope before the last line, because t1 borrows turtles immutably. When t1 gets dropped, it might re-use the borrow, so that needs to happen BEFORE turtles gets borrowed mutably.

        let new_turtle = {
            let t1 = self.turtles[t1_index].clone();
            let t2 = self.turtles[t2_index].clone();
            // let mut t1 = (*self.turtles[t1_index]).borrow_mut();
            // let mut t2 = (*self.turtles[t2_index]).borrow_mut();

            Rc::new(RefCell::new(Turtle::breed(t1, t2, name)))
        };

        self.turtles[t1_index].borrow_mut().add_child(new_turtle.clone());
        self.turtles[t2_index].borrow_mut().add_child(new_turtle.clone());

        self.turtles.push(new_turtle);
    }

    pub fn turtles(&self) -> std::slice::Iter<TurtleRef> {
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

    /**
     * The caller must not mutate the turtles in the returned vector!
     */
    pub fn turtles_with_name(&self, name: &String) -> Rc<Vec<TurtleRef>> {
        let mut borrowed_cache = self.turtle_name_cache.borrow_mut();

        if ! borrowed_cache.contains_key(name) {
            let mut result = Vec::new();
            for turtle in &self.turtles {
                if *turtle.borrow().name() == *name {
                    result.push(turtle.clone());
                }
            }

            let result_rc = Rc::new(result);
            borrowed_cache.insert(name.clone(), result_rc);
        }

        let res = borrowed_cache.get(name).expect("Name should be in the cache, since we just checked to make sure it's there.");
        res.clone()
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
