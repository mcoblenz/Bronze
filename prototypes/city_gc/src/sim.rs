use rand::prelude::*;
use crate::grid::*;
use bronze::GC;


pub fn step(grid: &mut Grid) {

    for r in grid.rows.iter_mut() {
        for t in r.iter_mut() {
            let mut rng = rand::thread_rng();
            let n = rng.gen_range(0, 5);
            if n == 0 {
                let new_tile = tile::ZonedTile::new();                
                let new_tile_box = Box::new(new_tile);
                *t = GC::new(new_tile_box);
            }
        }
    }
}

