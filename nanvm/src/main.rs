use io_impl::RealIo;
use io_trait::Io;
use nanvm_lib::parser::parse;
use nanvm_lib::parser::path::concat;
use nanvm_lib::{mem::local::Local, parser::Context};

fn main() {
    let local = Local::default();
    let io = RealIo();
    let path = "../../nanvm-lib/test/test_import_main.d.cjs";
    let context = Context::new(
        &local,
        &io,
        concat(io.current_dir().unwrap().as_str(), path),
    );
    let result = parse(&context);
    match result {
        Ok(_) => println!("ok"),
        Err(_) => print!("err"),
    }

    //todo:
    //1. read text file to string
    //2. print json tokens from the string
}
