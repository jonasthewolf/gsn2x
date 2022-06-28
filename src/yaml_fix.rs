use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use std::collections::BTreeMap;
use std::marker::PhantomData;

// Copied and adapted from https://serde.rs/deserialize-map.html
// to work around an issue in serde_yaml that does not check for duplicate keys in input YAML.
// Duplicate keys are no valid YAML but this is ignored by serde_yaml.

#[derive(Default, Debug, PartialEq)]
pub struct YamlFixMap<K, V>(BTreeMap<K, V>)
where
    K: Ord;

impl<K: Ord, V> YamlFixMap<K, V> {
    pub fn new() -> YamlFixMap<K, V> {
        YamlFixMap(BTreeMap::<K, V>::new())
    }

    pub fn into_inner(self) -> BTreeMap<K, V> {
        self.0
    }
}

struct YamlFixMapVisitor<K, V>
where
    K: Ord,
{
    marker: PhantomData<fn() -> YamlFixMap<K, V>>,
}

impl<K: Ord, V> YamlFixMapVisitor<K, V> {
    fn new() -> Self {
        YamlFixMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for YamlFixMapVisitor<K, V>
where
    K: Deserialize<'de> + Ord + std::fmt::Display,
    V: Deserialize<'de>,
{
    type Value = YamlFixMap<K, V>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with unique keys")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = YamlFixMap::new();

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            let errmsg = format!("Element {} is already existing", key);
            if map.0.insert(key, value).is_some() {
                return Err(serde::de::Error::custom(errmsg));
            }
        }

        Ok(map)
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, K, V> Deserialize<'de> for YamlFixMap<K, V>
where
    K: Deserialize<'de> + Ord + std::fmt::Display,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(YamlFixMapVisitor::new())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn format() {
        let btm = BTreeMap::<String, String>::new();
        let mm = YamlFixMap(btm.clone()).into_inner();
        assert_eq!(format!("{:?}", mm), format!("{:?}", btm));
    }
    #[test]
    fn debug() {
        assert!(YamlFixMap::<String, String>::new() == YamlFixMap::<String, String>::new());
    }
    #[test]
    fn duplicate() {
        let m = serde_yaml::from_str::<YamlFixMap<String, String>>("x:\n\nx:");
        assert!(m.is_err());
        assert_eq!(
            format!("{:?}", m),
            "Err(Message(\"Element x is already existing\", Some(Pos { marker: Marker { index: 1, line: 1, col: 1 }, path: \".\" })))"
        );
    }
    #[test]
    fn unknown_format() {
        let input = "- A\n\n- B\n\n- C\n";
        let res = serde_yaml::from_str::<YamlFixMap<String, String>>(input);
        assert!(res.is_err());
        assert_eq!(
            format!("{:?}", res),
            "Err(Message(\"invalid type: sequence, expected a map with unique keys\", Some(Pos { marker: Marker { index: 0, line: 1, col: 0 }, path: \".\" })))"
        );
    }
}
