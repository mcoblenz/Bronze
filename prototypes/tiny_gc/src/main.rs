use bronze::*;
use bronze_derive::*;

trait ATrait {
    fn doit(&self);
}

#[derive(Trace)]
struct AStruct {
    data: i32,
}

impl ATrait for AStruct {
    fn doit(&self) {
        println!("Hello, world!");
    }
}

fn main() {
    let a = Gc::new(AStruct {data: 42});
    let b = Gc::new(AStruct {data: 42});

}
