use std::{mem::size_of, ptr::write};

fn consume_vector(my: My) {
    let mut x: [u8; size_of::<My>()] = [0; size_of::<My>()];
    unsafe {
         write(&mut x as *mut u8 as *mut My, my);
    }
}

struct My {
    initialized: bool
}

impl My {
    fn new() -> Self{
        println!("created");
        Self { initialized: true }
    }
}

impl Drop for My {
    fn drop(&mut self) {
        self.initialized = false;
        println!("dropped");
    }
}

fn main() {
    println!("Hello world!{}",1);

    let my = My::new();
    println!("1");
    consume_vector(my);
    println!("2");

    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}