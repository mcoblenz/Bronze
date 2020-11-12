use rand::prelude::*;
use crate::grid::*;

pub fn step(grid: &Grid) {
    for t in grid.iter_mut() {
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(0, 10);
        if n == 0 {
            let new_tile = tile::ZonedTile::new();
            
            *t = Box::new(new_tile);
        }
    }    
}

