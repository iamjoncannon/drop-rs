pub mod macros;

pub fn pretty_printed_json(value: &str) -> Option<String> {
    let deserialization_result: Result<serde_json::Value, serde_json::Error> =
        serde_json::from_str(value);

    match deserialization_result {
        Ok(json) => {
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            Some(pretty)
        }
        Err(err) => {
            log::trace!("output invalid json {err:?}");
            None
        }
    }
}