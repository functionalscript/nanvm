use std::io::{self};
use io_trait::Io;

pub fn run(io: &impl Io) -> io::Result<()> {
    let mut a = io.args();
    a.next().unwrap();
    let input = a.next().unwrap();
    let output = a.next().unwrap();
    
    todo!()
}