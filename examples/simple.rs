use std::{cell::RefCell, rc::Rc};

use dirgraphsvg::{
    edges::EdgeType,
    nodes::{
    context::Context, new_goal, new_strategy, new_solution, new_justification, new_assumption,
    },
    DirGraph,
};

fn main() -> Result<(), std::io::Error> {
    let dg = DirGraph::default();
    let goal = Rc::new(RefCell::new(new_goal(
        "G1",
        "My extremely very, very, very, long Goal dEscription",
        false,
        None,
        None,
        None,
    )));
    let goal2 = Rc::new(RefCell::new(new_goal(
        "G2",
        "under lighted, undeveloped",
        true,
        None,
        None,
        None,
    )));
    let goal3 = Rc::new(RefCell::new(new_goal(
        "G3",
        "sub di dub di dub",
        false,
        None,
        None,
        Some(3),
    )));
    let goal4 = Rc::new(RefCell::new(new_goal(
        "G4",
        "circle di circle",
        false,
        None,
        None,
        None,
    )));
    let goal5 = Rc::new(RefCell::new(new_goal(
        "G5",
        "elcric id elcric",
        false,
        None,
        None,
        Some(2),
    )));
    let strategy = Rc::new(RefCell::new(new_strategy(
        "S1",
        "test strategy",
        false,
        None,
        None,
        None,
    )));
    let context = Rc::new(RefCell::new(Context::new("C1", "some context", None, None)));
    let solution = Rc::new(RefCell::new(new_solution(
        "Sn1",
        "test solution",
        None,
        None,
        None,
    )));
    let solution2 = Rc::new(RefCell::new(new_solution(
        "Sn2",
        "test another solution",
        None,
        None,
        Some(2),
    )));
    let solution3 = Rc::new(RefCell::new(new_solution(
        "Sn3",
        "yet another solution",
        None,
        None,
        None,
    )));
    let solution4 = Rc::new(RefCell::new(new_solution(
        "Sn4",
        "another forced solution",
        None,
        None,
        Some(1),
    )));
    let justification = Rc::new(RefCell::new(new_justification(
        "J1",
        "lalalsfa wrnasdf asdfa sdf asdlmösgm qwjsnf asndflan asdfa as",
        None,
        None,
        None,
    )));
    let assumption = Rc::new(RefCell::new(new_assumption(
        "A1",
        "teadskfasjdfjne",
        None,
        None,
        None,
    )));
    dg.set_font("Arial", 12.0)
        .set_size(1500, 1500)
        .add_node(justification.clone())
        .add_node(goal.clone())
        .add_node(goal2.clone())
        .add_node(goal3.clone())
        .add_node(goal4.clone())
        .add_node(goal5.clone())
        .add_node(strategy.clone())
        .add_node(assumption.clone())
        .add_node(context.clone())
        .add_node(solution.clone())
        .add_node(solution2.clone())
        .add_node(solution3.clone())
        .add_node(solution4.clone())
        .add_edge(goal.clone(), context.clone(), EdgeType::InContextOf)
        .add_edge(goal.clone(), solution2.clone(), EdgeType::SupportedBy)
        .add_edge(goal.clone(), strategy.clone(), EdgeType::SupportedBy)
        .add_edge(strategy.clone(), solution, EdgeType::SupportedBy)
        .add_edge(goal.clone(), justification, EdgeType::InContextOf)
        .add_edge(goal.clone(), assumption, EdgeType::InContextOf)
        .add_edge(strategy.clone(), goal2, EdgeType::SupportedBy)
        .add_edge(strategy, goal3, EdgeType::SupportedBy)
        .add_edge(goal.clone(), goal4.clone(), EdgeType::SupportedBy)
        .add_edge(goal4.clone(), solution3.clone(), EdgeType::SupportedBy)
        .add_edge(goal4, solution4.clone(), EdgeType::SupportedBy)
        .add_edge(goal5.clone(), solution3.clone(), EdgeType::SupportedBy)
        .add_edge(goal, goal5, EdgeType::SupportedBy)
        .write_to_file(std::path::Path::new("examples/simple.svg"))?;
    Ok(())
}
