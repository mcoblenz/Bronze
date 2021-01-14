use rand::prelude::*;
use crate::grid::*;
use bronze::Gc;


pub fn step(grid: &mut Grid) {

    for r in grid.rows.iter_mut() {
        for t in r.iter_mut() {
            let mut rng = rand::thread_rng();
            let n = rng.gen_range(0, 5);
            if n == 0 {
                let new_tile = tile::ZonedTile::new();                
                *t = Gc::new(new_tile);
            }
        }
    }
}

