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
    let output_data_type = file_to_data_type(&output);
    match output_data_type {
        Ok(data_type) => match parse(&mut context) {
            Ok(parse_result) => match data_type {
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
            Err(parse_error) => Err(Error::other(parse_error.to_string())),
        },
        Err(e) => Err(e),
    }
}

fn file_to_data_type(s: &str) -> Result<DataType, Error> {
    if s.ends_with(".json") {
        return Ok(DataType::Json);
    }
    if s.ends_with("d.cjs") {
        return Ok(DataType::Cjs);
    }
    if s.ends_with("d.mjs") {
        return Ok(DataType::Mjs);
    }
    Err(Error::other("invalid output extension"))
}

#[cfg(test)]
mod test {
    use io_test::VirtualIo;
    use io_trait::Io;
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::run;

    #[test]
    #[wasm_bindgen_test]
    fn test_json() {
        let io: VirtualIo = VirtualIo::new(&["test_json.json", "output.json"]);

        let main = include_str!("../../test/test-json.json");
        let main_path = "test_json.json";
        io.write(main_path, main.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.json").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"{"key":[true,false,null]}"#);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_output_data_type() {
        let io: VirtualIo = VirtualIo::new(&["test_json.json", "output.d.cjs"]);

        let main = include_str!("../../test/test-json.json");
        let main_path = "test_json.json";
        io.write(main_path, main.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.d.cjs").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"module.exports={"key":[true,false,null]}"#);

        let io: VirtualIo = VirtualIo::new(&["test_json.json", "output.d.mjs"]);

        let main = include_str!("../../test/test-json.json");
        let main_path = "test_json.json";
        io.write(main_path, main.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.d.mjs").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"export default {"key":[true,false,null]}"#);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_cjs() {
        let io: VirtualIo = VirtualIo::new(&["test_djs.d.cjs", "output.d.cjs"]);

        let main = include_str!("../../test/test-djs.d.cjs");
        let main_path = "test_djs.d.cjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.d.cjs").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"module.exports={"id":null}"#);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mjs() {
        let io: VirtualIo = VirtualIo::new(&["test_djs.d.mjs", "output.d.mjs"]);

        let main = include_str!("../../test/test-djs.d.mjs");
        let main_path = "test_djs.d.mjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.d.mjs").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"export default {"id":null}"#);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_cjs_import() {
        let io: VirtualIo = VirtualIo::new(&["test_import_main.d.cjs", "output.d.cjs"]);

        let main = include_str!("../../test/test_import_main.d.cjs");
        let main_path = "test_import_main.d.cjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.cjs");
        let module_path = "test_import_main.d.cjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.d.cjs").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"module.exports=3"#);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_mjs_import() {
        let io: VirtualIo = VirtualIo::new(&["test_import_main.d.mjs", "output.d.mjs"]);

        let main = include_str!("../../test/test_import_main.d.mjs");
        let main_path = "test_import_main.d.mjs";
        io.write(main_path, main.as_bytes()).unwrap();

        let module = include_str!("../../test/test_import_module.d.mjs");
        let module_path = "test_import_main.d.mjs";
        io.write(module_path, module.as_bytes()).unwrap();

        let result = run(&io);
        assert!(result.is_ok());
        let ouput_vec = io.read("output.d.mjs").unwrap();
        let vec = String::from_utf8(ouput_vec).unwrap();
        assert_eq!(vec, r#"export default 4"#);
    }
}
