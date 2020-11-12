mod grid;
mod sim;


fn main() {
    let grid = grid::Grid::new(4, 4);

    print!("{}", grid);
    sim::step(&grid);
    print!("{}", grid);    
}
