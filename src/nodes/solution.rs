use super::elliptical_node::EllipticalNode;

#[derive(Clone)]
pub struct Solution;

impl Solution {
    pub fn new(
        id: &str,
        text: &str,
        url: Option<String>,
        classes: Option<Vec<String>>,
    ) -> EllipticalNode {
        EllipticalNode::new(id, text, None, true, url, classes)
    }
}
