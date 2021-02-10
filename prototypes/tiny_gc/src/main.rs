use bronze::*;

trait ATrait {
    fn doit(&self);
}

struct AStruct {
    data: i32,
}

impl ATrait for AStruct {
    fn doit(&self) {
        println!("Hello, world!");
    }
}

impl GcTrace for AStruct {}


pub struct LittleBox<T: GcTrace + ?Sized + 'static> {
    data: T,
}

fn main() {
    let a = Gc::new(AStruct {data: 42});
    let b = Gc::new(AStruct {data: 42});

}
