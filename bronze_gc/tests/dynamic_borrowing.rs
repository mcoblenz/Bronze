use bronze_gc::*;
use bronze_derive::{Trace, Finalize};
use serial_test::serial;

#[derive(Trace, Finalize)]
pub struct IntContainer {
    x: i32,
}

#[test]
#[serial]
fn borrow_twice() {
    let ic1 = Gc::new(IntContainer {x: 2});
    let _ic1_borrow = ic1.borrow();
    let ic1_borrow2 = ic1.borrow();
    assert!(ic1_borrow2.x == 2);
}

// Commented out because this test intentionally fails to compile.
// #[test]
// #[serial]
// fn invalid_borrow_1() {
//     let mut ic1 = Gc::new(IntContainer {x: 2});
//     let _ic1_borrow = ic1.borrow();
//     let _ic1_borrow2 = ic1.borrow_mut(); // Compile error: outstanding immutable borrow
// }

#[test]
#[serial]
#[should_panic]
fn invalid_borrow_2() {
    let ic1 = Gc::new(IntContainer {x: 2});
    let mut ic2 = ic1;
    let _ic1_borrow = ic1.borrow();
    let _ic2_borrow_mut = ic2.borrow_mut(); // should panic! Outstanding immutable borrow.
}

#[test]
#[serial]
#[should_panic]
fn invalid_borrow_3() {
    let mut ic1 = Gc::new(IntContainer {x: 2});
    let ic2 = ic1;
    let _ic1_borrow = ic1.borrow_mut();
    let _ic2_borrow_mut = ic2.borrow(); // should panic! Outstanding mutable borrow.
}

#[test]
#[serial]
#[should_panic]
fn invalid_borrow_4() {
    let mut ic1 = Gc::new(IntContainer {x: 2});
    let mut ic2 = ic1;
    let _ic1_borrow: GcBorrowMut<IntContainer> = ic1.borrow_mut();
    let _ic2_borrow = ic2.borrow_mut(); // should panic! Outstanding mutable borrow.
}


#[test]
#[serial]
fn valid_borrow_scope() {
    let ic1 = Gc::new(IntContainer {x: 2});
    let mut ic2 = ic1;
    {
        let _ic1_borrow = ic1.borrow();
    }
    let _ic2_borrow_mut = ic2.borrow_mut(); // OK because prior borrow is out of scope.
}

#[test]
#[serial]
fn aliases() {
    let mut ic1 = Gc::new(IntContainer {x: 2});
    let ic2 = ic1;
    {
        let mut ic1_borrow_mut = ic1.borrow_mut();
        (*ic1_borrow_mut).x = 42;
    }
    let ic2_borrow = ic2.borrow(); // OK because prior borrow is out of scope.
    assert!(ic2_borrow.x == 42);
}

#[test]
#[serial]
fn mutability() {
    let x: GcRef<u32> = Gc::new(42);
    // *ic = 43 would fail to compile because ic isn't mutable!
    // However, we can generate a mutable alias:
    let mut x2 = x;
    *x2 = 43;

    assert!(*x2 == 43);
}


pub fn set(mut c: GcRef<IntContainer>, x: i32) {
  c.x = x;
}

#[test]
#[serial]
pub fn make_two_references() {
  let c1 = GcRef::new(IntContainer{x: 42});
  let c2 = c1;
  // c1 and c2 both reference the same object.

  set(c2, 42);
  set(c1, 43);
  // Now both reference an object with value 43.
}

fn take_borrowed_ic(r: &IntContainer) {
    println!("{}", r.x);
}

fn take_two_borrow_mut_ics(r1: &mut IntContainer, r2: &mut IntContainer) {
    println!("{}", r1.x);
}

#[test]
#[serial]
pub fn make_reference() {
    let c1 = GcRef::new(IntContainer{x: 42});
    take_borrowed_ic(&c1.borrow());
}


#[test]
#[serial]
#[should_panic]
pub fn make_mutable_references() {
    let mut c1 = GcRef::new(IntContainer{x: 42});
    let mut c2 = c1;

    take_two_borrow_mut_ics(&mut c1.borrow_mut(), &mut c2.borrow_mut());
}

#[test]
#[serial]
pub fn make_reference_let() {
    let c1 = GcRef::new(IntContainer{x: 42});
    let icr: &IntContainer = &c1.borrow();
    take_borrowed_ic(icr);
}