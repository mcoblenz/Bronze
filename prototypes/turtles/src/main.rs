mod campus;
mod turtle;

use crate::turtle::Turtle;
use crate::campus::Campus;

use rand::prelude::*;



fn main() {
    let mut campus = Campus::new(7);
    println!("{:?}", campus);

    let mut rng = rand::thread_rng();

    for _i in 0..5 {
        let t1_index = rng.gen_range(0..campus.size());
        let mut t2_index = rng.gen_range(0..campus.size());
        
        // Turtles can't breed with themselves.
        while t2_index == t1_index {
            t2_index = rng.gen_range(0..campus.size());
        }

        campus.breed_turtles(t1_index, t2_index);

        println!("{:?}", campus);
    }
}
