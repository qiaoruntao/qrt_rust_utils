#[cfg(test)]
mod test_basic {
    use std::collections::HashMap;

    use qrt_rust_macros::ToHashMap;

    #[derive(ToHashMap)]
    struct TestStruct {
        name: String,
        number: u32,
    }

    #[test]
    fn test_into() {
        let test_struct = TestStruct { name: "".to_string(), number: 123 };
        let x: HashMap<String, String> = test_struct.into();
        assert_eq!(x.get("name").unwrap().as_str(), "");
        assert_eq!(x.get("number").unwrap().as_str(), "123");
    }
}

#[cfg(test)]
mod test_serde_basic {
    use std::collections::HashMap;

    use qrt_rust_macros::ToHashMap;

    #[derive(ToHashMap)]
    struct TestStruct2 {
        #[serde(rename = "name2")]
        name: String,
        number: u32,
    }

    #[test]
    fn test() {
        let test_struct = TestStruct2 { name: "".to_string(), number: 123 };
        let x: HashMap<String, String> = test_struct.into();
        assert_eq!(x.get("name"), None);
        assert_eq!(x.get("name2").unwrap().as_str(), "");
        assert_eq!(x.get("number").unwrap().as_str(), "123");
    }
}