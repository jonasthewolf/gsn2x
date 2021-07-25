use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::Serialize;
use std::collections::BTreeMap;
use std::marker::PhantomData;

// Copied and adapted from https://serde.rs/deserialize-map.html

#[derive(Debug, Serialize)]
pub struct MyMap<K, V>(BTreeMap<K, V>)
where
    K: Ord;

impl<K: Ord, V> MyMap<K, V> {
    pub fn new() -> MyMap<K, V> {
        MyMap(BTreeMap::<K, V>::new())
    }
}

impl<K, V> std::ops::Deref for MyMap<K, V>
where
    K: Ord,
{
    type Target = BTreeMap<K, V>;
    fn deref(&self) -> &BTreeMap<K, V> {
        &self.0
    }
}

impl<K, V> std::ops::DerefMut for MyMap<K, V>
where
    K: Ord,
{
    fn deref_mut(&mut self) -> &mut BTreeMap<K, V> {
        &mut self.0
    }
}

struct MyMapVisitor<K, V>
where
    K: Ord,
{
    marker: PhantomData<fn() -> MyMap<K, V>>,
}

impl<K: Ord, V> MyMapVisitor<K, V> {
    fn new() -> Self {
        MyMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for MyMapVisitor<K, V>
where
    K: Deserialize<'de> + Ord + std::fmt::Display,
    V: Deserialize<'de>,
{
    type Value = MyMap<K, V>;

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
        let mut map = MyMap::new();

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
impl<'de, K, V> Deserialize<'de> for MyMap<K, V>
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
        deserializer.deserialize_map(MyMapVisitor::new())
    }
}
