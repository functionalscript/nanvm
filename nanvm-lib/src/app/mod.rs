use io_trait::Io;
use std::io::{self, Error};

use crate::{
    common::default::default,
    mem::global::GLOBAL,
    parser::{parse, Context, DataType},
    serializer::{to_djs::to_djs, to_json::to_json},
};

pub fn run(io: &impl Io) -> io::Result<()> {
    let mut a = io.args();
    a.next().unwrap();
    let input = a.next().unwrap();
    let output = a.next().unwrap();

    let mc = &mut default();
    let mut context = Context::new(GLOBAL, io, input, mc);
    match parse(&mut context) {
        Ok(parse_result) => match parse_result.data_type {
            DataType::Json => {
                let to_json_result = to_json(parse_result.any);
                match to_json_result {
                    Ok(s) => io.write(&output, s.as_bytes()),
                    Err(e) => Err(Error::other(e)),
                }
            }
            DataType::Cjs => {
                let to_json_result = to_djs(parse_result.any, true);
                match to_json_result {
                    Ok(s) => io.write(&output, s.as_bytes()),
                    Err(e) => Err(Error::other(e)),
                }
            }
            DataType::Mjs => {
                let to_json_result = to_djs(parse_result.any, false);
                match to_json_result {
                    Ok(s) => io.write(&output, s.as_bytes()),
                    Err(e) => Err(Error::other(e)),
                }
            }
            _ => unreachable!(),
        },
        Err(_) => Err(Error::other("parse error")),
    }
}
