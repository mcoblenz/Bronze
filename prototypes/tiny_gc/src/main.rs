use bronze::*;

trait ATrait {
    fn doit(&self);
}

struct AStruct {}

impl ATrait for AStruct {
    fn doit(&self) {
        println!("Hello, world!");
    }
}

impl GcTrace for AStruct {}


fn main() {
    let b = Gc::new(AStruct {});
    b.as_ref().doit();

}
