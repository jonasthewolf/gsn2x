use std::{cell::RefCell, ops::BitOr};

use svg::{
    Node,
    node::element::{Element, Group, Path, Use, path::Data},
};

use crate::{
    dirgraph::{DirectedGraph, DirectedGraphEdgeType, EdgeDecorator},
    dirgraphsvg::{
        escape_text,
        render::{BOTTOM_RIGHT_CORNER, PADDING_HORIZONTAL, PADDING_VERTICAL, TOP_LEFT_CORNER},
        util::curve::CubicBezierCurve,
    },
    gsn::GsnEdgeType,
};

use super::{
    DirGraph,
    layout::Margin,
    nodes::{Port, SvgNode},
    render::{ACP_BOX_SIZE, BOTTOM_LEFT_CORNER, TOP_RIGHT_CORNER, create_text},
    util::{font::str_line_bounding_box, point2d::Point2D},
};

///
/// The EdgeType in one direction
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleEdge<'a> {
    InContextOf,
    SupportedBy,
    Composite,
    ChallengesNode,
    ChallengesRelation(&'a str),
}

impl<'a> BitOr for SingleEdge<'a> {
    type Output = SingleEdge<'a>;

    fn bitor(self, rhs: Self) -> Self::Output {
        match self {
            SingleEdge::InContextOf => {
                if rhs == SingleEdge::InContextOf {
                    SingleEdge::InContextOf
                } else {
                    SingleEdge::Composite
                }
            }
            SingleEdge::SupportedBy => {
                if rhs == SingleEdge::SupportedBy {
                    SingleEdge::SupportedBy
                } else {
                    SingleEdge::Composite
                }
            }
            SingleEdge::Composite => SingleEdge::Composite,
            _ => self, // Ignore others
        }
    }
}

///
/// The edge type used for rendering
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdgeType<'a> {
    OneWay(SingleEdge<'a>),
    TwoWay((SingleEdge<'a>, SingleEdge<'a>)),
}

impl DirectedGraphEdgeType<'_> for EdgeType<'_> {
    fn is_primary_child_edge(&self) -> bool {
        matches!(
            *self,
            EdgeType::OneWay(SingleEdge::SupportedBy)
                | EdgeType::OneWay(SingleEdge::Composite)
                | EdgeType::TwoWay((SingleEdge::SupportedBy, _))
                | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
                | EdgeType::TwoWay((SingleEdge::Composite, _))
                | EdgeType::TwoWay((_, SingleEdge::Composite))
        )
    }

    fn is_secondary_child_edge(&self) -> bool {
        matches!(*self, EdgeType::OneWay(SingleEdge::InContextOf))
    }

    fn is_inverted_child_edge(&self) -> bool {
        matches!(*self, EdgeType::OneWay(SingleEdge::ChallengesNode))
            || matches!(*self, EdgeType::OneWay(SingleEdge::ChallengesRelation(_)))
    }
}

///
/// Convert from GsnEdgeType to EdgeType
///
impl<'a> From<GsnEdgeType<'a>> for EdgeType<'a> {
    fn from(value: GsnEdgeType<'a>) -> Self {
        match value {
            GsnEdgeType::SupportedBy => Self::OneWay(SingleEdge::SupportedBy),
            GsnEdgeType::InContextOf => Self::OneWay(SingleEdge::InContextOf),
            GsnEdgeType::ChallengesNode => Self::OneWay(SingleEdge::ChallengesNode),
            GsnEdgeType::ChallengesRelation(r) => Self::OneWay(SingleEdge::ChallengesRelation(r)),
        }
    }
}

///
/// Height of the arrow
///
const MARKER_HEIGHT: u32 = 10;

///
/// Render a single edge
///
pub(super) fn render_edge(
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    render_graph: &DirGraph,
    ranks: &[Vec<Vec<&str>>],
    bounding_boxes: &[Vec<[Point2D<i32>; 4]>],
    source: &str,
    target: &(String, EdgeType),
    width: i32,
) -> Vec<Element> {
    let curve_points = match target.1 {
        EdgeType::OneWay(SingleEdge::ChallengesRelation(relation_target)) => {
            // Render edge from source to mid point of target.0 and relation_target edge
            let source_id = &target.0;
            let target_id = relation_target;
            let other_edge = get_edge_points(
                graph,
                render_graph,
                ranks,
                bounding_boxes,
                source_id,
                (target_id, target.1),
                width,
            );
            let p = get_mid_point(&other_edge);
            let dummy_target = RefCell::new(SvgNode::new_dummy(p.x, p.y));
            create_curve_points(
                render_graph,
                bounding_boxes,
                EdgeType::OneWay(SingleEdge::ChallengesRelation(target_id)),
                width,
                (
                    graph.get_nodes().get(source).unwrap().borrow(),
                    ranks
                        .iter()
                        .position(|x| x.iter().flatten().any(|&v| v == source_id))
                        .unwrap(),
                ),
                (
                    dummy_target.borrow(),
                    ranks
                        .iter()
                        .position(|x| x.iter().flatten().any(|&v| v == relation_target))
                        .unwrap(),
                ),
            )
        }
        _ => {
            // Render edge between source and target.0 nodes
            let source_id = source;
            get_edge_points(
                graph,
                render_graph,
                ranks,
                bounding_boxes,
                source_id,
                (&target.0, target.1),
                width,
            )
        }
    };

    let data = create_path_data_for_points(&curve_points);
    let arrow_end_id = match &target.1 {
        EdgeType::OneWay(SingleEdge::InContextOf)
        | EdgeType::TwoWay((_, SingleEdge::InContextOf)) => Some("url(#incontextof_arrow)"),
        EdgeType::OneWay(SingleEdge::SupportedBy)
        | EdgeType::TwoWay((_, SingleEdge::SupportedBy)) => Some("url(#supportedby_arrow)"),
        EdgeType::OneWay(SingleEdge::Composite) | EdgeType::TwoWay((_, SingleEdge::Composite)) => {
            Some("url(#composite_arrow)")
        }
        EdgeType::OneWay(SingleEdge::ChallengesNode) => Some("url(#challenges_arrow)"),
        EdgeType::OneWay(SingleEdge::ChallengesRelation(_)) => Some("url(#challenges_arrow)"),
        _ => unreachable!(),
    };
    let arrow_start_id = match &target.1 {
        EdgeType::TwoWay((SingleEdge::InContextOf, _)) => Some("url(#incontextof_arrow)"),
        EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => Some("url(#supportedby_arrow)"),
        EdgeType::TwoWay((SingleEdge::Composite, _)) => Some("url(#composite_arrow)"),
        _ => None,
    };
    let mut classes = "gsnedge".to_string();
    match target.1 {
        EdgeType::OneWay(SingleEdge::InContextOf)
        | EdgeType::TwoWay((_, SingleEdge::InContextOf))
        | EdgeType::TwoWay((SingleEdge::InContextOf, _)) => classes.push_str(" gsninctxt"),
        EdgeType::OneWay(SingleEdge::SupportedBy)
        | EdgeType::TwoWay((_, SingleEdge::SupportedBy))
        | EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => classes.push_str(" gsninspby"),
        EdgeType::OneWay(SingleEdge::Composite) | EdgeType::TwoWay((_, SingleEdge::Composite)) => {
            // Already covered by all other matches
            //| EdgeType::TwoWay((SingleEdge::Composite, _))
            classes.push_str(" gsncomposite")
        }
        EdgeType::OneWay(SingleEdge::ChallengesNode)
        | EdgeType::OneWay(SingleEdge::ChallengesRelation(_)) => classes.push_str(" gsnchllngs"),
        _ => unreachable!(),
    };
    let mut e = Path::new()
        .set("d", data)
        .set("fill-opacity", "0")
        .set("stroke", "black")
        .set("stroke-width", 1u32);
    if let Some(arrow_id) = arrow_end_id {
        e = e.set("marker-end", arrow_id);
    }
    if let Some(arrow_id) = arrow_start_id {
        e = e.set("marker-start", arrow_id);
    }
    if matches!(target.1, EdgeType::OneWay(SingleEdge::ChallengesNode))
        || matches!(
            target.1,
            EdgeType::OneWay(SingleEdge::ChallengesRelation(_))
        )
    {
        e = e.set("stroke-dasharray", "5 5");
    }
    e = e.set("class", classes);
    let mut result: Vec<Element> = vec![e.into()];
    if let Some(EdgeDecorator::Defeated) = graph.get_edge_decorator(source, &target.0) {
        let p = get_mid_point(&curve_points);
        result.push(
            Use::new()
                .set("href", "#defeated_cross")
                .set("x", p.x - 10)
                .set("y", p.y - 10)
                .into(),
        );
    }
    result.extend(render_acps(
        graph,
        render_graph,
        source,
        target,
        curve_points,
    ));
    result
}

///
/// Get points of the edge.
///
///
fn get_edge_points(
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    render_graph: &DirGraph<'_>,
    ranks: &[Vec<Vec<&str>>],
    bounding_boxes: &[Vec<[Point2D<i32>; 4]>],
    source_id: &str,
    target: (&str, EdgeType),
    width: i32,
) -> Vec<(Point2D<i32>, Point2D<i32>)> {
    let s = graph.get_nodes().get(source_id).unwrap().borrow();
    let t = graph.get_nodes().get(target.0).unwrap().borrow();
    let s_rank = ranks
        .iter()
        .position(|x| x.iter().flatten().any(|&v| v == source_id))
        .unwrap();
    let t_rank = ranks
        .iter()
        .position(|x| x.iter().flatten().any(|&v| v == target.0))
        .unwrap();

    create_curve_points(
        render_graph,
        bounding_boxes,
        target.1,
        width,
        (s, s_rank),
        (t, t_rank),
    )
}

///
/// Create curve points between two nodes
///
///
fn create_curve_points(
    render_graph: &DirGraph<'_>,
    bounding_boxes: &[Vec<[Point2D<i32>; 4]>],
    edge_type: EdgeType<'_>,
    width: i32,
    source: (std::cell::Ref<'_, SvgNode>, usize), /* Node, rank */
    target: (std::cell::Ref<'_, SvgNode>, usize),
) -> Vec<(Point2D<i32>, Point2D<i32>)> {
    let s_pos = source.0.get_position();
    let t_pos = target.0.get_position();

    let (marker_start_height, marker_end_height, support_distance) = match edge_type {
        EdgeType::OneWay(_) => (0i32, MARKER_HEIGHT as i32, 3i32 * MARKER_HEIGHT as i32),
        EdgeType::TwoWay(_) => (
            MARKER_HEIGHT as i32,
            MARKER_HEIGHT as i32,
            3i32 * MARKER_HEIGHT as i32,
        ),
    };

    let (start, start_sup, end, end_sup) = get_start_and_end_points(
        source.0,
        target.0,
        marker_start_height,
        support_distance,
        marker_end_height,
    );
    let mut curve_points = vec![(start, start_sup)];
    add_supporting_points(
        &mut curve_points,
        bounding_boxes,
        &(target.1, t_pos),
        &(source.1, s_pos),
        width,
        &render_graph.margin,
    );
    curve_points.push((end, end_sup));
    curve_points
}

///
/// Render an Assurance Claim Point (ACP)
///
///
fn render_acps(
    graph: &DirectedGraph<'_, RefCell<SvgNode>, EdgeType>,
    render_graph: &DirGraph<'_>,
    source: &str,
    target: &(String, EdgeType),
    curve_points: Vec<(Point2D<i32>, Point2D<i32>)>,
) -> Option<Element> {
    if let Some(EdgeDecorator::Acps(acps)) = graph.get_edge_decorator(source, &target.0) {
        let mut svg_acp = Group::new()
            .set(
                "id",
                format!(
                    "acp_{}_{}_{}",
                    escape_text(source).to_lowercase(),
                    escape_text(&target.0).to_lowercase(),
                    escape_text(&acps.join("_")).to_lowercase()
                ),
            )
            .set("class", "gsnacp");
        let center_segment = (curve_points.len() - 1) / 2;
        let curve = CubicBezierCurve::new(
            curve_points[center_segment].0,
            curve_points[center_segment].1,
            curve_points[center_segment + 1].1,
            curve_points[center_segment + 1].0,
        );
        let coords = curve.get_coordinates_for_t(0.5);
        let turning_vector = curve.get_first_derivative_for_t(0.5).normalize();
        let acp_text = acps.join(", ");
        let acp_x = coords.x - ACP_BOX_SIZE;
        let acp_y = coords.y - ACP_BOX_SIZE;
        let acp_text_bb = str_line_bounding_box(&render_graph.font, &acp_text, false);
        let acp_x_text = coords.x
            + ((ACP_BOX_SIZE + PADDING_HORIZONTAL) as f64 * turning_vector.y) as i32
            - ((1.0 - turning_vector.y) * ((acp_text_bb.0) as f64 / 2.0)) as i32;
        let acp_y_text = coords.y
            + ((acp_text_bb.1 + PADDING_VERTICAL) as f64 * turning_vector.x) as i32
            + ((1.0 - turning_vector.x) * ACP_BOX_SIZE as f64) as i32;
        svg_acp.append(
            Use::new()
                .set("href", "#acp")
                .set("x", acp_x)
                .set("y", acp_y),
        );
        svg_acp.append(create_text(
            &acp_text.into(),
            acp_x_text,
            acp_y_text,
            &render_graph.font,
            false,
        ));
        Some(svg_acp.into())
    } else {
        None
    }
}

///
/// Create bezier curves based on the calculated supporting points.
///
fn create_path_data_for_points(curve_points: &[(Point2D<i32>, Point2D<i32>)]) -> Data {
    let parameters = vec![
        curve_points[0].1.x as f32, // start supporting point
        curve_points[0].1.y as f32,
        curve_points[1].1.x as f32, // end supporting point
        curve_points[1].1.y as f32,
        curve_points[1].0.x as f32, // end point
        curve_points[1].0.y as f32,
    ];

    let mut data = Data::new()
        .move_to((curve_points[0].0.x, curve_points[0].0.y))
        .cubic_curve_to(parameters);
    for point in curve_points.iter().skip(2).take(curve_points.len() - 1) {
        let parameters = vec![
            point.1.x as f32, // end supporting point
            point.1.y as f32,
            point.0.x as f32, // end point
            point.0.y as f32,
        ];

        data = data.smooth_cubic_curve_to(parameters)
    }
    data
}

///
/// If at least a rank is skipped, add supporting points.
///
fn add_supporting_points(
    curve_points: &mut Vec<(Point2D<i32>, Point2D<i32>)>,
    bounding_boxes: &[Vec<[Point2D<i32>; 4]>],
    target: &(usize, Point2D<i32>), // target (rank, position)
    source: &(usize, Point2D<i32>), // source (rank, position)
    width: i32,
    margin: &Margin,
) {
    // If a rank is skipped, test if we hit anything.
    // If not, everything is fine. If so, we need to add supporting points to the curve.
    // The supporting point is closest to its predecessor point not hitting anything.
    // Y is the median of the ranks y positions.
    // This way we get one supporting point for each skipped rank.
    for bboxes in bounding_boxes
        .iter()
        .take(std::cmp::max(source.0, target.0))
        .skip(std::cmp::min(source.0, target.0) + 1)
    {
        if bboxes
            .iter()
            .any(|bbox| is_line_intersecting_with_box(&source.1, &target.1, bbox))
        {
            // Find the empty spaces on each skipped rank.
            // For each empty space create at least three potential points incl. supporting points.
            // Minimize by the distance to the last point in the curve and the distance in x-direction to the target node
            let first_in_rank = first_free_center_point(bboxes.first().unwrap());
            let last_in_rank = last_free_center_point(bboxes.last().unwrap(), width);
            let mut boxes = vec![first_in_rank];
            boxes.append(&mut bboxes.to_vec());
            boxes.push(last_in_rank);
            let last_point = curve_points.last().unwrap(); // unwrap ok, since start point is added before first call to this function.
            let best_free_points = boxes
                .windows(2)
                .flat_map(|window| {
                    get_potential_supporting_points(window, target.1.x, target.0 > source.0, margin)
                })
                .min_by_key(|(p, _)| {
                    (p.distance(&last_point.0) as f64
                        * f64::sqrt((target.1.x - p.x) as f64 * (target.1.x - p.x) as f64))
                        as i32
                })
                .unwrap();

            curve_points.push(best_free_points);
        }
    }
    // Reverse order if t_rank < s_rank
    if target.0 < source.0 {
        let tmp_first = curve_points.remove(0);
        curve_points.reverse();
        curve_points.insert(0, tmp_first);
    }
}

///
/// Get three points to choose from for supporting point
///
fn get_potential_supporting_points(
    window: &[[Point2D<i32>; 4]],
    target_x: i32,
    top_down: bool,
    margin: &Margin,
) -> Vec<(Point2D<i32>, Point2D<i32>)> {
    let y = (window[0][TOP_RIGHT_CORNER].y
        + window[0][BOTTOM_RIGHT_CORNER].y
        + window[1][TOP_LEFT_CORNER].y
        + window[1][BOTTOM_LEFT_CORNER].y)
        / 4;
    // Make the support point either further up or further down, depending on direction of the edge
    let supporting_y = if top_down {
        std::cmp::min(window[0][TOP_RIGHT_CORNER].y, window[1][TOP_LEFT_CORNER].y) as f64
            - 2.0 * margin.top as f64
    } else {
        std::cmp::max(window[0][TOP_RIGHT_CORNER].y, window[1][TOP_LEFT_CORNER].y) as f64
            + 2.0 * margin.bottom as f64
    } as i32;

    let mut points: Vec<_> = vec![
        (
            (window[0][TOP_RIGHT_CORNER].x, y),
            (window[0][TOP_RIGHT_CORNER].x, supporting_y),
        ),
        (
            (
                (window[0][TOP_RIGHT_CORNER].x + window[1][TOP_LEFT_CORNER].x) / 2,
                y,
            ),
            (
                (window[0][TOP_RIGHT_CORNER].x + window[1][TOP_LEFT_CORNER].x) / 2,
                supporting_y,
            ),
        ),
    ]
    .into_iter()
    .collect();
    if window[0][TOP_RIGHT_CORNER].x < target_x && window[1][TOP_LEFT_CORNER].x > target_x {
        points.push(((target_x, y), (target_x, supporting_y)));
    }
    points.push((
        (window[1][TOP_LEFT_CORNER].x, y),
        (window[1][TOP_LEFT_CORNER].x, supporting_y),
    ));
    points
        .into_iter()
        .map(|(p1, p2)| (p1.into(), p2.into()))
        .collect()
}

///
/// Add the area from 0 to the first bounding box
///
fn first_free_center_point(bbox: &[Point2D<i32>; 4]) -> [Point2D<i32>; 4] {
    let p = Point2D {
        x: 0,
        y: (bbox[TOP_LEFT_CORNER].y + bbox[BOTTOM_LEFT_CORNER].y) / 2,
    };
    [p, p, p, p]
}

///
/// Add the area from the last bounding box to the edge of the document
///
fn last_free_center_point(bbox: &[Point2D<i32>; 4], width: i32) -> [Point2D<i32>; 4] {
    let p = Point2D {
        x: width,
        y: (bbox[TOP_RIGHT_CORNER].y + bbox[BOTTOM_RIGHT_CORNER].y) / 2,
    };
    [p, p, p, p]
}

///
/// Get start and end points of the edge
///
fn get_start_and_end_points(
    s: std::cell::Ref<'_, SvgNode>,
    t: std::cell::Ref<'_, SvgNode>,
    marker_start_height: i32,
    support_distance: i32,
    marker_end_height: i32,
) -> (Point2D<i32>, Point2D<i32>, Point2D<i32>, Point2D<i32>) {
    let s_pos = s.get_position();
    let t_pos = t.get_position();
    let (start, start_sup, end, end_sup) =
        if s_pos.y + s.get_height() / 2 < t_pos.y - t.get_height() / 2 {
            (
                s.get_coordinates(Port::South) + (0, marker_start_height),
                s.get_coordinates(Port::South) + (0, support_distance),
                t.get_coordinates(Port::North) + (0, -marker_end_height),
                t.get_coordinates(Port::North) + (0, -support_distance),
            )
        } else if s_pos.y - s.get_height() / 2 > t_pos.y + t.get_height() / 2 {
            (
                s.get_coordinates(Port::North) + (0, -marker_start_height),
                s.get_coordinates(Port::North) + (0, -support_distance),
                t.get_coordinates(Port::South) + (0, marker_end_height),
                t.get_coordinates(Port::South) + (0, support_distance),
            )
        } else if s_pos.x - s.get_width() / 2 > t_pos.x + t.get_width() / 2 {
            (
                s.get_coordinates(Port::West) + (-marker_start_height, 0),
                s.get_coordinates(Port::West) + (-support_distance, 0),
                t.get_coordinates(Port::East) + (marker_end_height, 0),
                t.get_coordinates(Port::East) + (support_distance, 0),
            )
        } else {
            (
                s.get_coordinates(Port::East) + (marker_start_height, 0),
                s.get_coordinates(Port::East) + (support_distance, 0),
                t.get_coordinates(Port::West) + (-marker_end_height, 0),
                t.get_coordinates(Port::West) + (-support_distance, 0),
            )
        };
    (start, start_sup, end, end_sup)
}

///
/// Get center point of edge
///
fn get_mid_point(curve_points: &[(Point2D<i32>, Point2D<i32>)]) -> Point2D<i32> {
    let center_segment = (curve_points.len() - 1) / 2;
    let curve = CubicBezierCurve::new(
        curve_points[center_segment].0,
        curve_points[center_segment].1,
        curve_points[center_segment + 1].1,
        curve_points[center_segment + 1].0,
    );
    curve.get_coordinates_for_t(0.5)
}

///
/// Check if line is hitting a box
/// Algorithm from https://stackoverflow.com/a/293052/2516756
///
fn is_line_intersecting_with_box(
    start: &Point2D<i32>,
    end: &Point2D<i32>,
    bbox: &[Point2D<i32>; 4],
) -> bool {
    let line = |x: i32, y: i32| -> i32 {
        ((end.y - start.y) as f64 * x as f64
            + (start.x - end.x) as f64 * y as f64
            + (end.x * start.y - start.x * end.y) as f64) as i32
    };
    if bbox.iter().all(|bb| line(bb.x, bb.y) < 0) || bbox.iter().all(|bb| line(bb.x, bb.y) > 0) {
        false
    } else {
        !((start.x > bbox[TOP_RIGHT_CORNER].x && end.x > bbox[TOP_RIGHT_CORNER].x)
            || (start.x < bbox[BOTTOM_LEFT_CORNER].x && end.x < bbox[BOTTOM_LEFT_CORNER].x)
            || (start.y > bbox[TOP_RIGHT_CORNER].y && end.y > bbox[TOP_RIGHT_CORNER].y)
            || (start.y < bbox[BOTTOM_LEFT_CORNER].y && end.y < bbox[BOTTOM_LEFT_CORNER].y))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cloning() {
        let si = SingleEdge::Composite;
        assert_eq!(si.clone(), si);
    }

    #[test]
    fn merging() {
        let si1 = SingleEdge::InContextOf;
        let si2 = SingleEdge::SupportedBy;
        let si3 = SingleEdge::Composite;
        assert_eq!(si1 | si1, si1);
        assert_eq!(si1 | si2, si3);
        assert_eq!(si1 | si3, si3);
        assert_eq!(si2 | si2, si2);
        assert_eq!(si2 | si3, si3);
        assert_eq!(si3 | si3, si3);
    }

    #[test]
    fn formatting() {
        assert_eq!(
            format!("{:?}", EdgeType::OneWay(SingleEdge::InContextOf)),
            "OneWay(InContextOf)"
        );
    }
}
