use dirgraphsvg::{
    edges::EdgeType,
    nodes::{new_assumption, new_context, new_goal, new_justification, new_solution, new_strategy},
    DirGraph,
};

fn main() -> Result<(), std::io::Error> {
    let dg = DirGraph::default();
    let goal = new_goal(
        "G1",
        "My extremely very, very, very, long Goal dEscription",
        false,
        None,
        None,
        None,
    );
    let goal2 = new_goal(
        "G2",
        "undeveloped undeveloped undeveloped",
        true,
        None,
        None,
        None,
    );
    let goal3 = new_goal("G3", "sub di dub di dub", false, None, None, Some(3));
    let goal4 = new_goal("G4", "circle di circle", false, None, None, None);
    let goal5 = new_goal("G5", "elcric id elcric", false, None, None, Some(2));
    let strategy = new_strategy("S1", "test strategy", false, None, None, None);
    let context = new_context("C1", "some context", None, None);
    let solution = new_solution("Sn1", "test solution", None, None, None);
    let solution2 = new_solution("Sn2", "test another solution", None, None, Some(2));
    let solution3 = new_solution("Sn3", "yet another solution", None, None, None);
    let solution4 = new_solution("Sn4", "another forced solution", None, None, None);
    let justification = new_justification(
        "J1",
        "lalalsfa wrnasdf asdfa sdf asdlmösgm qwjsnf asndflan asdfa as",
        None,
        None,
    );
    let assumption = new_assumption("A1", "teadskfasjdfjne", None, None);
    let justification2 = new_justification(
        "J2",
        "asdfgasgnajkg aksdnnglert klnalsdn kölnsdg ljsmdg snnjk slls qwjsnf asndflan asdfa as",
        None,
        None,
    );
    dg.set_font("Arial", 12.0)
        .add_node(justification.clone())
        .add_node(goal.clone())
        .add_node(goal2.clone())
        .add_node(justification2.clone())
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
        .add_edge(goal5.clone(), solution4.clone(), EdgeType::SupportedBy)
        .add_edge(goal5.clone(), solution3.clone(), EdgeType::SupportedBy)
        .add_edge(goal, goal5.clone(), EdgeType::SupportedBy)
        .add_edge(goal5.clone(), justification2, EdgeType::InContextOf)
        .write_to_file(std::path::Path::new("examples/simple.svg"))?;
    Ok(())
}
