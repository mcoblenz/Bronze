mod campus;
mod turtle;
mod cookbook;
mod genetics;
mod magic;

use crate::turtle::Turtle;
use crate::campus::Campus;
use crate::cookbook::Cookbook;

use rand::prelude::*;



fn main() {
    let mut campus = Campus::new(2);

    // Can't call campus_size within call to gen_range because both calls need to borrow campus.
    let campus_size = campus.size();
    println!("{:?}", campus);

    let mut rng = rand::thread_rng();

    println!("Breeding more turtles");
    for _i in 0..2 {
        let t1_index = rng.gen_range(0..campus_size);
        let mut t2_index = rng.gen_range(0..campus_size);
        
        // Turtles can't breed with themselves.
        while t2_index == t1_index {
            t2_index = rng.gen_range(0..campus_size);
        }

        {
            campus.breed_turtles(t1_index, t2_index);
        }
    }

    println!("{:?}", campus);


    // Ask each turtle to choose a favorite recipe.
    // let cookbook = Cookbook::new();
    // for t in campus.turtles() {
    //     let recipe = t.borrow().choose_recipe(&cookbook);
    //     println!("{:?}'s favorite recipe is {:?}", t, recipe);
    // }

    println!("sending turtles to school");
    campus.school();
    println!("{:?}", campus);

}
