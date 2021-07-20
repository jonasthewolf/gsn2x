use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GsnYamlNode {
    text: String,
    pub(crate) in_context_of: Option<Vec<String>>,
    pub(crate) supported_by: Option<Vec<String>>,
}

pub fn validate(nodes: &BTreeMap<String, GsnYamlNode>) {
    let mut wnodes: HashSet<String> = nodes.keys().cloned().collect();
    for (key, node) in nodes {
        validate_id(key);
        validate_reference(&nodes, key, &node);
        // Remove all keys if they are referenced
        if let Some(context) = node.in_context_of.as_ref() {
            for cnode in context {
                wnodes.remove(cnode);
            }
        }
        if let Some(support) = node.supported_by.as_ref() {
            for snode in support {
                wnodes.remove(snode);
            }
        }
    }
    if wnodes.len() > 1 {
        error!(
            "Error: There is more than one unreferenced element: {:?}",
            wnodes
        );
    }
}

fn validate_id(id: &String) {
    if !(id.starts_with("Sn")
        || id.starts_with('G')
        || id.starts_with('A')
        || id.starts_with('J')
        || id.starts_with('S')
        || id.starts_with('C'))
    {
        error!(
            "Error: Elememt {} is of unknown type. Please see README for supported types",
            id
        );
    }
}

fn check_references(
    nodes: &BTreeMap<String, GsnYamlNode>,
    node: &str,
    refs: &[String],
    diag: &str,
) {
    let mut set = HashSet::with_capacity(refs.len());
    let wrong_refs: Vec<&String> = refs
        .iter()
        .inspect(|&n| {
            if !set.insert(n) {
                warn!(
                    "Warning: Element {} has duplicate entry {} in {}.",
                    node, n, diag
                );
            }
        })
        .filter(|&n| !nodes.contains_key(n))
        .collect();
    for wref in wrong_refs {
        error!("Error: Element {} has unresolved {}: {}", node, diag, wref);
    }
}

fn validate_reference(nodes: &BTreeMap<String, GsnYamlNode>, key: &String, node: &GsnYamlNode) {
    if let Some(context) = node.in_context_of.as_ref() {
        check_references(nodes, key, context, "context");
    }
    if let Some(support) = node.supported_by.as_ref() {
        check_references(nodes, key, support, "supported by element");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use env_logger::{Builder, WriteStyle};
    use log::LevelFilter;
    use std::{
        io,
        sync::mpsc::{channel, Sender},
    };

    struct WriteAdapter {
        sender: Sender<u8>,
    }

    impl io::Write for WriteAdapter {
        // On write we forward each u8 of the buffer to the sender and return the length of the buffer
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for chr in buf {
                self.sender.send(*chr).unwrap();
            }
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn init_logger(output: Sender<u8>) {
        // There can be only one logger, thus, the need for try_init()
        let _ = Builder::new()
            .filter(None, LevelFilter::Info)
            .write_style(WriteStyle::Always)
            .format_timestamp(None)
            .target(env_logger::Target::Pipe(Box::new(WriteAdapter {
                sender: output,
            })))
            .is_test(true)
            .try_init();
    }

    #[test]
    fn unknown_id() {
        let (rx, tx) = channel();
        init_logger(rx);
        validate_id(&"X1".to_owned());
        assert_eq!(
            std::str::from_utf8(&tx.try_iter().collect::<Vec<u8>>()).unwrap(),
            "[ERROR gsn2x::gsn] Error: Elememt X1 is of unknown type. Please see README for supported types\n"
        );
    }

    #[test]
    fn known_id() {
        let (rx, tx) = channel();
        init_logger(rx);
        validate_id(&"Sn1".to_owned());
        assert_eq!(
            std::str::from_utf8(&tx.try_iter().collect::<Vec<u8>>()).unwrap(),
            ""
        );
    }
}
