use std::{cell::RefCell, rc::Rc};

use dirgraphsvg::{
    edges::EdgeType,
    nodes::{
        context::Context, goal::Goal, justification::Justification, solution::Solution,
        strategy::Strategy,
    },
    DirGraph,
};

fn main() -> Result<(), std::io::Error> {
    let dg = DirGraph::default();
    let goal = Rc::new(RefCell::new(Goal::new(
        "G1",
        "My extremely very, very, very, long Goal dEscription",
        false,
        None,
        None,
    )));
    let goal2 = Rc::new(RefCell::new(Goal::new(
        "G2",
        "under lighted, undeveloped",
        true,
        None,
        None,
    )));
    let strategy = Rc::new(RefCell::new(Strategy::new(
        "S1",
        "test strategy",
        false,
        None,
        None,
    )));
    let context = Rc::new(RefCell::new(Context::new("C1", "some context", None, None)));
    let solution = Rc::new(RefCell::new(Solution::new(
        "Sn1",
        "test solution",
        None,
        None,
    )));
    let justification = Rc::new(RefCell::new(Justification::new(
        "J1",
        "lalalsfa wrnasdf asdfa sdf asdlmösgm qwjsnf asndflan asdfa as",
        None,
        None,
    )));
    dg.set_font("Arial", 12.0)
        .set_size(1500, 1500)
        .add_node(goal.clone())
        .add_node(goal2.clone())
        .add_node(strategy.clone())
        .add_node(context.clone())
        .add_node(solution.clone())
        .add_node(justification.clone())
        .add_edge(goal.clone(), context.clone(), EdgeType::InContextOf)
        .add_edge(goal.clone(), strategy.clone(), EdgeType::SupportedBy)
        .add_edge(strategy.clone(), solution, EdgeType::SupportedBy)
        .add_edge(strategy, goal2, EdgeType::SupportedBy)
        .add_edge(goal, justification, EdgeType::InContextOf)
        .write_to_file(std::path::Path::new("examples/simple.svg"))?;
    Ok(())
}
