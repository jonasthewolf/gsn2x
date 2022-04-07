use std::{cell::RefCell, rc::Rc};

use svg::node::element::{Link, Group};

use crate::{util::point2d::Point2D, FontInfo};

use self::{
    away_node::{AwayNode, AwayType},
    box_node::BoxNode,
    context_node::ContextNode,
    elliptical_node::EllipticalNode,
};

pub mod away_node;
pub mod box_node;
pub mod context_node;
pub mod elliptical_node;
pub(crate) mod invisible_node;

#[derive(PartialEq)]
pub enum Port {
    North,
    East,
    South,
    West,
}

pub trait Node {
    fn get_id(&self) -> &str;
    fn calculate_size(&mut self, font: &FontInfo, suggested_char_wrap: u32);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn set_position(&mut self, pos: &Point2D);
    fn get_position(&self) -> Point2D;
    fn get_coordinates(&self, port: &Port) -> Point2D;
    fn get_forced_level(&self) -> Option<usize>;
    fn set_forced_level(&mut self, level: usize);
    fn render(&mut self, font: &FontInfo) -> svg::node::element::Group;
}

pub(crate) fn get_port_default_coordinates(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    port: &Port,
) -> Point2D {
    Point2D {
        x: match port {
            Port::North => x,
            Port::East => x + width / 2,
            Port::South => x,
            Port::West => x - width / 2,
        },
        y: match port {
            Port::North => y - height / 2,
            Port::East => y,
            Port::South => y + height / 2,
            Port::West => y,
        },
    }
}

///
/// 
/// 
/// 
/// 
fn setup_basics(id: &str, classes: Option<Vec<String>>, url: Option<String>) -> Group {
    let mut g = Group::new().set("id", format!("node_{}", crate::util::escape_text(&id)));
    if let Some(classes) = &classes {
        g = g.set("class", classes.join(" "))
    }
    if let Some(url) = &url {
        let link = Link::new();
        g = g.add(link.set("xlink:href", url.as_str()));
    }
    g
}

pub fn new_assumption(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<EllipticalNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnasmp".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(EllipticalNode::new(
        id,
        text,
        Some("A".to_owned()),
        false,
        url,
        Some(new_classes),
    )))
}

pub fn new_away_assumption(
    id: &str,
    text: &str,
    module: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawayasmp".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        AwayType::Assumption,
        url,
        Some(new_classes),
    )))
}

pub fn new_justification(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<EllipticalNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnjust".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(EllipticalNode::new(
        id,
        text,
        Some("J".to_owned()),
        false,
        url,
        Some(new_classes),
    )))
}

pub fn new_away_justification(
    id: &str,
    text: &str,
    module: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawayjust".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        AwayType::Justification,
        url,
        Some(new_classes),
    )))
}

pub fn new_solution(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<EllipticalNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnsltn".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(EllipticalNode::new(
        id,
        text,
        None,
        true,
        url,
        Some(new_classes),
    )))
}

pub fn new_away_solution(
    id: &str,
    text: &str,
    module: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawaysltn".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        AwayType::Solution,
        url,
        Some(new_classes),
    )))
}

pub fn new_strategy(
    id: &str,
    text: &str,
    undeveloped: bool,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<BoxNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnstgy".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(BoxNode::new(
        id,
        text,
        undeveloped,
        15,
        url,
        Some(new_classes),
    )))
}

pub fn new_goal(
    id: &str,
    text: &str,
    undeveloped: bool,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<BoxNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsngoal".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(BoxNode::new(
        id,
        text,
        undeveloped,
        0,
        url,
        Some(new_classes),
    )))
}

pub fn new_away_goal(
    id: &str,
    text: &str,
    module: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawaygoal".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        AwayType::Goal,
        url,
        Some(new_classes),
    )))
}

pub fn new_context(
    id: &str,
    text: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<ContextNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnctxt".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(ContextNode::new(
        id,
        text,
        url,
        Some(new_classes),
    )))
}

pub fn new_away_context(
    id: &str,
    text: &str,
    module: &str,
    url: Option<String>,
    classes: Option<Vec<String>>,
) -> Rc<RefCell<AwayNode>> {
    let mut new_classes: Vec<String> = vec!["gsnelem".to_owned(), "gsnawayctxt".to_owned()];
    if let Some(classes) = classes {
        classes.into_iter().for_each(|c| new_classes.push(c));
    }
    Rc::new(RefCell::new(AwayNode::new(
        id,
        text,
        module,
        AwayType::Context,
        url,
        Some(new_classes),
    )))
}
