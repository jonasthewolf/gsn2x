use super::elliptical_node::EllipticalNode;

#[derive(Clone)]
pub struct Justification;

impl Justification {
    pub fn new(
        id: &str,
        text: &str,
        url: Option<String>,
        classes: Option<Vec<String>>,
        forced_level: Option<usize>,
    ) -> EllipticalNode {
        EllipticalNode::new(
            id,
            text,
            Some("J".to_owned()),
            false,
            url,
            classes,
            forced_level,
        )
    }
}
