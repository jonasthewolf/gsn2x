use super::elliptical_node::EllipticalNode;

#[derive(Clone)]
pub struct Assumption;

impl Assumption {
    pub fn new(
        id: &str,
        text: &str,
        url: Option<String>,
        classes: Option<Vec<String>>,
    ) -> EllipticalNode {
        EllipticalNode::new(id, text, Some("A".to_owned()), false, url, classes)
    }
}
