use super::box_node::BoxNode;

#[derive(Clone)]
pub struct Goal;

impl Goal {
    pub fn new(
        id: &str,
        text: &str,
        undeveloped: bool,
        url: Option<String>,
        classes: Option<Vec<String>>,
    ) -> BoxNode {
        BoxNode::new(id, text, undeveloped, 0, url, classes)
    }
}
