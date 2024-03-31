use io_impl::RealIo;
use io_trait::Io;
use nanvm_lib::common::default::default;
use nanvm_lib::parser::parse;
use nanvm_lib::parser::path::concat;
use nanvm_lib::serializer::WriteJson;
use nanvm_lib::{mem::local::Local, parser::Context};

fn parser_test() {
    let local = Local::default();
    let io = RealIo();
    let path = "nanvm-lib/test/test_import_main.d.cjs";
    let mut mc = default();
    let mut context = Context::new(
        &local,
        &io,
        concat(io.current_dir().unwrap().as_str(), path),
        &mut mc,
    );
    let result = parse(&mut context);
    match result {
        Ok(parse_result) => {
            let mut s = String::new();
            s.write_json(parse_result.any).unwrap();
            println!("ok {}", s);
        }
        Err(err) => print!("err {:?}", err),
    }

    let local = Local::default();
    let io = RealIo();
    let path = "nanvm-lib/test/test_import_main.d.mjs";
    let mut mc = default();
    let mut context = Context::new(
        &local,
        &io,
        concat(io.current_dir().unwrap().as_str(), path),
        &mut mc,
    );
    let result = parse(&mut context);
    match result {
        Ok(parse_result) => {
            let mut s = String::new();
            s.write_json(parse_result.any).unwrap();
            println!("ok {}", s);
        }
        Err(err) => print!("err {:?}", err),
    }
}

fn main() {
    parser_test();
}
