mod grid;
mod sim;


fn main() {
    let mut grid = grid::Grid::new(4, 4);

    print!("initial grid:");
    print!("{}", grid);
    for _i in 0..10 {
        sim::step(&mut grid);
        print!("{}", grid);    
    }
}
