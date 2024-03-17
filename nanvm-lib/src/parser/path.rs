pub fn normalize(path: &str) -> String {
    let path_split: Vec<_> = path.split('/').collect();
    let mut result_split: Vec<&str> = Vec::new();
    for &dir in path_split.iter() {
        match dir {
            ".." => {
                let last = result_split.last();
                match last {
                    Some(x) if x != &".." => {
                        result_split.pop();
                    }
                    _ => {
                        result_split.push(dir);
                    }
                }
            }
            _ => {
                result_split.push(dir);
            }
        }
    }
    result_split.join("/")
}

pub fn concat(_a: &str, _b: &str) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::parser::path::normalize;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let norm = normalize("../../dir/file.json");
        assert_eq!(norm, "../../dir/file.json");

        let norm = normalize("../../dir/../file.json");
        assert_eq!(norm, "../../file.json");
    }
}
