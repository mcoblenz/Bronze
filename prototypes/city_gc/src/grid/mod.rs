// A Grid represents a plot of land, divided into squares. 
// It is of fixed size.
use std::fmt;
use bronze::GCRef;
use bronze::GC;

pub mod tile;

pub struct Grid {
    pub rows: Vec<Vec<GCRef<dyn tile::Tile>>>
}

impl Grid {
    pub fn new(num_rows: usize, num_cols: usize) -> Grid {
        let mut rows: Vec<Vec<GCRef<dyn tile::Tile>>> = Vec::new();

        for _r in 0..num_rows {
            // Interestingly, if I remove this type annotation, the line below won't typecheck!
            // ^^^ expected trait object `dyn grid::tile::Tile`, found struct `grid::tile::EmptyTile`
            let mut row: Vec<GCRef<dyn tile::Tile>> = Vec::new();
            for _c in 0..num_cols {
                row.push(GC::new(Box::new(tile::EmptyTile{})));
            }
            rows.push(row);
        }

        Grid{rows}
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first = false;
        
        for r in &self.rows {
            if !is_first {
                writeln!(f, "")?;
            }
            if is_first {
                is_first = false;
            }
            for c in r {
                let s = c.to_string();
                write!(f, "{}", s)?;
            }
        }
        write!(f, "\n")
    }
}
