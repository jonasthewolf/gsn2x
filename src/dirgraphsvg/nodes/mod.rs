use std::{cell::RefCell, rc::Rc};

use svg::node::element::{Element, Group, Link};

use crate::dirgraphsvg::{util::point2d::Point2D, FontInfo};

use self::{
    away_node::{AwayNode, AwayType},
    box_node::BoxNode,
    context_node::ContextNode,
    elliptical_node::EllipticalNode,
};

use super::util::{escape_node_id, escape_url};

pub mod away_node;
pub mod box_node;
pub mod context_node;
pub mod elliptical_node;

#[derive(Eq, PartialEq)]
pub enum Port {
    North,
    East,
    South,
    West,
}

pub trait Node {
    fn get_id(&self) -> &str;
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32);
    fn get_width(&self) -> i32;
    fn get_height(&self) -> i32;
    fn set_position(&mut self, pos: &Point2D);
    fn get_position(&self) -> Point2D;
    fn get_coordinates(&self, port: &Port) -> Point2D;
    fn render(&mut self, font: &FontInfo) -> svg::node::element::Element;
}

///
///
///
///
pub(crate) fn get_port_default_coordinates(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    port: &Port,
) -> Point2D {
    Point2D {
        x: match port {
            Port::North => x,
            Port::East => x + (width / 2) as i32,
            Port::South => x,
            Port::West => x - (width / 2) as i32,
        },
        y: match port {
            Port::North => y - (height / 2) as i32,
            Port::East => y,
            Port::South => y + (height / 2) as i32,
            Port::West => y,
        },
    }
}

///
///
///
///
///
pub(crate) fn setup_basics(id: &str, classes: &[String], url: &Option<String>) -> Element {
    let mut g = Group::new().set("id", escape_node_id(id));
    g = g.set("class", classes.join(" "));
    if let Some(url) = &url {
        let link = Link::new();
        link.set("href", escape_url(url.as_str())).add(g).into()
    } else {
        g.into()
    }
}

///
///
///
///
pub fn new_assumption(
    id: &str,
    text: &str,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<EllipticalNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnasmp".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(EllipticalNode::new(
        id,
        text,
        Some("A".to_owned()),
        false,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_away_assumption(
    id: &str,
    text: &str,
    module: &str,
    module_url: Option<String>,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawayasmp".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        module_url,
        AwayType::Assumption,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_justification(
    id: &str,
    text: &str,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<EllipticalNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnjust".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(EllipticalNode::new(
        id,
        text,
        Some("J".to_owned()),
        false,
        url,
        new_classes,
    )))
}

///
///
///
///
///
pub fn new_away_justification(
    id: &str,
    text: &str,
    module: &str,
    module_url: Option<String>,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawayjust".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        module_url,
        AwayType::Justification,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_solution(
    id: &str,
    text: &str,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<EllipticalNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnsltn".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(EllipticalNode::new(
        id,
        text,
        None,
        true,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_away_solution(
    id: &str,
    text: &str,
    module: &str,
    module_url: Option<String>,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawaysltn".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        module_url,
        AwayType::Solution,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_strategy(
    id: &str,
    text: &str,
    undeveloped: bool,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<BoxNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnstgy".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(BoxNode::new(
        id,
        text,
        undeveloped,
        15,
        false,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_goal(
    id: &str,
    text: &str,
    undeveloped: bool,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<BoxNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsngoal".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(BoxNode::new(
        id,
        text,
        undeveloped,
        0,
        false,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_away_goal(
    id: &str,
    text: &str,
    module: &str,
    module_url: Option<String>,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawaygoal".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        module_url,
        AwayType::Goal,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_context(
    id: &str,
    text: &str,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<ContextNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnctxt".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(ContextNode::new(id, text, url, new_classes)))
}

///
///
///
///
pub fn new_away_context(
    id: &str,
    text: &str,
    module: &str,
    module_url: Option<String>,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawayctxt".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        module_url,
        AwayType::Context,
        url,
        new_classes,
    )))
}

///
///
///
///
pub fn new_module(
    id: &str,
    text: &str,
    url: Option<String>,
    mut classes: Vec<String>,
) -> Rc<RefCell<BoxNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnmodule".to_owned()];
    new_classes.append(&mut classes);
    Rc::new(RefCell::new(BoxNode::new(
        id,
        text,
        false,
        0,
        true,
        url,
        new_classes,
    )))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_module() {
        let m = new_module(
            "id",
            "text",
            Some("url".to_owned()),
            vec!["classa".to_owned()],
        );
        assert_eq!(m.borrow().get_id(), "id");
    }

    #[test]
    fn test_setup_basics() {
        let b = setup_basics("myid", &[], &None);
        assert_eq!(b.get_attributes()["id"].to_string(), "node_myid".to_owned());
    }
}
