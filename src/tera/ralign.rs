use std::collections::HashMap;

use tera::{Error, Filter, Result, Value};

pub struct Ralign;

impl Filter for Ralign {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
        ralign(
            value.to_string(),
            args.get("width")
                .ok_or_else(|| Error::msg("Parameter width missing"))?
                .as_u64()
                .ok_or_else(|| Error::msg("Parameter width is not an integer"))?,
        )
    }
}

///
/// Right align `s` by `width`.
///
fn ralign(s: String, width: u64) -> Result<Value> {
    Ok(Value::from(format!(
        "{:>width$}",
        s,
        width = width as usize
    )))
}

#[cfg(test)]
mod test {
    use super::*;
    use tera::ErrorKind;

    #[test]
    fn no_width() {
        let ww = Ralign {};
        let map = HashMap::<String, Value>::new();
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter width missing"
            )
        );
    }

    #[test]
    fn invalid_width() {
        let ww = Ralign {};
        let mut map = HashMap::<String, Value>::new();
        map.insert("width".to_owned(), Value::String("xyz".to_owned()));
        assert!(
            matches!(ww.filter(&Value::String("Test".to_owned()), &map).err().unwrap().kind, ErrorKind::Msg(t) if t == "Parameter width is not an integer")
        );
    }
}
