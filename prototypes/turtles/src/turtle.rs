use std::rc::Rc;
use std::cell::RefCell;


use crate::cookbook::*;
use crate::genetics::*;
use crate::magic::*;


#[derive(Debug, Copy, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8
}



impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        Color {r, g, b}
    }

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
    name: String,
    walking_speed: u32,
    color: Color,
    favorite_flavor: Flavor,

    // Can't just have a vector of children because the children are owned by the campus and they can't have two owners.
    children: Vec<Rc<RefCell<Turtle>>>,

    magical_item: Option<Box<dyn TurtlePower>>,
}


impl Turtle {
    // You normally don't create a new Turtle from nothing; instead, breed
    // two Turtles.
    // Lifetimes are critical and tricky here!
    // The output lifetime doesn't depend on the lifetimes
    // of the parameters.
    pub fn breed(p1: Rc<RefCell<Turtle>>, p2: Rc<RefCell<Turtle>>, name: String) -> Turtle {
        let t1 = p1.borrow();
        let t2 = p2.borrow();

        Turtle {
            name,
            walking_speed: cross32(t1.walking_speed, t2.walking_speed),
            color: Color::cross(&t1.color, &t2.color),
            favorite_flavor: Flavor::random_flavor(),
            children: Vec::new(),
            magical_item: None,
        }
    }

    pub fn walking_speed(&self) -> u32 {
        self.walking_speed
    }

    pub fn set_walking_speed(&mut self, new_speed: u32) {
        self.walking_speed = new_speed;
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn favorite_flavor(&self) -> Flavor {
        self.favorite_flavor
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn new(name: String, walking_speed: u32, favorite_flavor: Flavor, color: Color) -> Turtle {
        Turtle {
            name, walking_speed, color, favorite_flavor, children: vec![], magical_item: None
        }
    }

    pub fn choose_recipe<'a>(&self, cookbook: &'a Cookbook) -> Option<&'a Recipe> {
        for recipe in cookbook.recipes() {
            if recipe.flavor() == self.favorite_flavor() {
                return Some(recipe);
            }
        }
        None
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Turtle>>) {
        self.children.push(child);
    }

    pub fn teach_children(&mut self) {
        for child in &self.children {
            let mut t = child.borrow_mut();
            let old_speed = t.walking_speed();
            t.set_walking_speed(old_speed + 1);
        }
    }

    pub fn set_color(&mut self, new_color: Color) {
        self.color = new_color;
    }


    pub fn take_magical_item(&mut self, item: Box<dyn TurtlePower>) {
        self.magical_item = Some(item);
    }
}
