mod campus;
mod turtle;

use crate::turtle::Turtle;
use crate::campus::Campus;




fn main() {
    let campus = Campus::new(7);
    println!("{:?}", campus);
}
