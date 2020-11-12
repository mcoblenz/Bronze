// A Grid represents a plot of land, divided into squares. 
// It is of fixed size.
use std::fmt;

pub mod tile;

pub struct Grid {
    rows: Vec<Vec<Box<dyn tile::Tile>>>
}

impl Grid {
    pub fn new(num_rows: usize, num_cols: usize) -> Grid {
        let mut rows: Vec<Vec<Box<dyn tile::Tile>>> = Vec::new();

        for _r in 0..num_rows {
            // Interestingly, if I remove this type annotation, the line below won't typecheck!
            // ^^^ expected trait object `dyn grid::tile::Tile`, found struct `grid::tile::EmptyTile`
            let mut row: Vec<Box<dyn tile::Tile>> = Vec::new();
            for _c in 0..num_cols {
                row.push(Box::new(tile::EmptyTile{}));
            }
            rows.push(row);
        }

        Grid{rows}
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {next_col: 0, next_row: 0, grid: self}
    }
}

struct IntoIter {
    next_col: usize,
    next_row: usize,
    grid: Grid,
}

impl Iterator for IntoIter {
    type Item = Box<dyn tile::Tile>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // If we're already past the end, stop.
        if self.next_row >= self.grid.rows.len() {
            return None;
        }
        // Otherwise, we should be within the grid.
        let current_row = self.grid.rows[self.next_row];
        let row_len = current_row.len();
        let result = current_row[self.next_col];

        self.next_col = self.next_col + 1;

        // Update indices
        if self.next_col >= row_len {
            self.next_col = 0;
            self.next_row = self.next_row + 1;
        }

        return Some(result);
    }
}

impl IntoIterator for Grid {
    type Item = Box<dyn tile::Tile>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter{next_col: 0, next_row: 0, grid: self}
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
            for c in &*r {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}


pub struct IterMut<'a> {
    // TODO
    // row_iter: std::slice::IterMut<'a, Box<dyn tile::Tile>>,
    
    next_col: usize,
    next_row: usize,
    grid: &'a mut Grid,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Box<dyn tile::Tile>;

    fn next(&mut self) -> Option<&'a mut Box<dyn tile::Tile>> {
        // If we're already past the end, stop.
        if self.next_row >= self.grid.rows.len() {
            return None;
        }
        // Otherwise, we should be within the grid.
        let current_row: &Vec<Box<dyn tile::Tile>> = &self.grid.rows[self.next_row];
        let row_len = current_row.len();
        let result = &mut current_row[self.next_col];

        self.next_col = self.next_col + 1;

        // Update indices
        if self.next_col >= row_len {
            self.next_col = 0;
            self.next_row = self.next_row + 1;
        }

        return Some(result);
    }
}
