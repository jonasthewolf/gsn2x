use std::{cell::RefCell, ops::BitOr};

use svg::node::element::{path::Data, Path};

use crate::{
    dirgraph::DirectedGraph,
    dirgraphsvg::render::{BOTTOM_RIGHT_CORNER, TOP_LEFT_CORNER},
    gsn::GsnEdgeType,
};

use super::{
    nodes::{Port, SvgNode},
    render::{BOTTOM_LEFT_CORNER, TOP_RIGHT_CORNER},
    util::point2d::Point2D,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SingleEdge {
    InContextOf,
    SupportedBy,
    Composite,
}

impl BitOr for SingleEdge {
    type Output = SingleEdge;

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
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdgeType {
    OneWay(SingleEdge),
    TwoWay((SingleEdge, SingleEdge)),
    // Invisible,
}

impl From<&GsnEdgeType> for EdgeType {
    fn from(value: &GsnEdgeType) -> Self {
        match value {
            GsnEdgeType::SupportedBy => Self::OneWay(SingleEdge::SupportedBy),
            GsnEdgeType::InContextOf => Self::OneWay(SingleEdge::InContextOf),
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
    ranks: &[Vec<Vec<&str>>],
    bounding_boxes: &[Vec<[Point2D; 4]>],
    source: &str,
    target: &str,
    edge_type: &EdgeType,
    width: i32,
) -> Path {
    let s = graph.get_nodes().get(source).unwrap().borrow();
    let s_rank = ranks
        .iter()
        .position(|x| x.iter().flatten().any(|&v| v == source))
        .unwrap();
    let t = graph.get_nodes().get(target).unwrap().borrow();
    let t_rank = ranks
        .iter()
        .position(|x| x.iter().flatten().any(|&v| v == target))
        .unwrap();
    let s_pos = s.get_position();
    let t_pos = t.get_position();

    let (marker_start_height, marker_end_height, support_distance) = match edge_type {
        EdgeType::OneWay(_) => (0i32, MARKER_HEIGHT as i32, 3i32 * MARKER_HEIGHT as i32),
        EdgeType::TwoWay(_) => (
            MARKER_HEIGHT as i32,
            MARKER_HEIGHT as i32,
            3i32 * MARKER_HEIGHT as i32,
        ),
    };

    let (start, start_sup, end, end_sup) = get_start_and_end_points(
        s,
        t,
        marker_start_height,
        support_distance,
        marker_end_height,
    );
    let mut curve_points = vec![(start, start_sup)];
    add_supporting_points(
        &mut curve_points,
        bounding_boxes,
        t_rank,
        s_rank,
        &s_pos,
        &t_pos,
        width,
    );
    curve_points.push((end, end_sup));

    let data = create_path_data_for_points(&curve_points);
    let arrow_end_id = match &edge_type {
        EdgeType::OneWay(SingleEdge::InContextOf)
        | EdgeType::TwoWay((_, SingleEdge::InContextOf)) => Some("url(#incontextof_arrow)"),
        EdgeType::OneWay(SingleEdge::SupportedBy)
        | EdgeType::TwoWay((_, SingleEdge::SupportedBy)) => Some("url(#supportedby_arrow)"),
        EdgeType::OneWay(SingleEdge::Composite) | EdgeType::TwoWay((_, SingleEdge::Composite)) => {
            Some("url(#composite_arrow)")
        }
    };
    let arrow_start_id = match &edge_type {
        EdgeType::TwoWay((SingleEdge::InContextOf, _)) => Some("url(#incontextof_arrow)"),
        EdgeType::TwoWay((SingleEdge::SupportedBy, _)) => Some("url(#supportedby_arrow)"),
        EdgeType::TwoWay((SingleEdge::Composite, _)) => Some("url(#composite_arrow)"),
        _ => None,
    };
    let mut classes = "gsnedge".to_string();
    match edge_type {
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
    };
    let mut e = Path::new()
        .set("d", data)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1u32);
    if let Some(arrow_id) = arrow_end_id {
        e = e.set("marker-end", arrow_id);
    }
    if let Some(arrow_id) = arrow_start_id {
        e = e.set("marker-start", arrow_id);
    }
    e = e.set("class", classes);
    e
}

///
///
///
fn create_path_data_for_points(curve_points: &[(Point2D, Point2D)]) -> Data {
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
///
///
fn add_supporting_points(
    curve_points: &mut Vec<(Point2D, Point2D)>,
    bounding_boxes: &[Vec<[Point2D; 4]>],
    t_rank: usize,
    s_rank: usize,
    s_pos: &Point2D,
    t_pos: &Point2D,
    width: i32,
) {
    // If a rank is skipped, test if we hit anything.
    // If not, everything is fine. If so, we need to add supporting points to the curve.
    // The supporting point is closest to its predecessor point not hitting anything.
    // Y is the median of the ranks y positions.
    // This way we get one supporting point for each skipped rank.
    for bboxes in bounding_boxes
        .iter()
        .take(std::cmp::max(s_rank, t_rank))
        .skip(std::cmp::min(s_rank, t_rank) + 1)
    {
        if bboxes
            .iter()
            .any(|bbox| is_line_intersecting_with_box(s_pos, t_pos, bbox))
        {
            let first_in_rank = first_free_center_point(bboxes.first().unwrap());
            let last_in_rank = last_free_center_point(bboxes.last().unwrap(), width);
            let mut boxes = vec![first_in_rank];
            boxes.append(&mut bboxes.to_vec());
            boxes.push(last_in_rank);
            let last_point = curve_points.last().unwrap(); // unwrap ok, since start point is added before first call to this function.
            let best_free_points = boxes
                .windows(2)
                .flat_map(|window| {
                    get_potential_supporting_points(window, t_pos.x, t_rank > s_rank)
                })
                .min_by_key(|(p, _)| {
                    (distance(p, &last_point.0) as f64
                        * f64::sqrt((t_pos.x - p.x) as f64 * (t_pos.x - p.x) as f64))
                        as i32
                })
                .unwrap();

            curve_points.push(best_free_points);
        }
    }
    // Reverse order if t_rank < s_rank
    if t_rank < s_rank {
        let tmp_first = curve_points.remove(0);
        curve_points.reverse();
        curve_points.insert(0, tmp_first);
    }
}

///
/// Get three points to choose from for supporting point
///
fn get_potential_supporting_points(
    window: &[[Point2D; 4]],
    target_x: i32,
    top_down: bool,
) -> Vec<(Point2D, Point2D)> {
    let y = (window[0][TOP_RIGHT_CORNER].y
        + window[0][BOTTOM_RIGHT_CORNER].y
        + window[1][TOP_LEFT_CORNER].y
        + window[1][BOTTOM_LEFT_CORNER].y)
        / 4;
    let supporting_y = if top_down {
        0.7 * (window[0][TOP_RIGHT_CORNER].y + window[1][TOP_LEFT_CORNER].y) as f64 / 2.0
    } else {
        1.2 * (window[0][BOTTOM_RIGHT_CORNER].y + window[1][BOTTOM_LEFT_CORNER].y) as f64 / 2.0
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
///
///
fn first_free_center_point(bbox: &[Point2D; 4]) -> [Point2D; 4] {
    let p = Point2D {
        x: 0,
        y: (bbox[TOP_LEFT_CORNER].y + bbox[BOTTOM_LEFT_CORNER].y) / 2,
    };
    [p, p, p, p]
}

///
///
///
fn last_free_center_point(bbox: &[Point2D; 4], width: i32) -> [Point2D; 4] {
    let p = Point2D {
        x: width,
        y: (bbox[TOP_RIGHT_CORNER].y + bbox[BOTTOM_RIGHT_CORNER].y) / 2,
    };
    [p, p, p, p]
}

///
///
/// TODO Choose port based on angle between direct connection
///
fn get_start_and_end_points(
    s: std::cell::Ref<'_, SvgNode>,
    t: std::cell::Ref<'_, SvgNode>,
    marker_start_height: i32,
    support_distance: i32,
    marker_end_height: i32,
) -> (Point2D, Point2D, Point2D, Point2D) {
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
///
/// Algorithm from https://stackoverflow.com/a/293052/2516756
///
fn is_line_intersecting_with_box(start: &Point2D, end: &Point2D, bbox: &[Point2D; 4]) -> bool {
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

///
/// Get the distance between two points
///
fn distance(p1: &Point2D, p2: &Point2D) -> i32 {
    f64::sqrt(
        (p1.x - p2.x) as f64 * (p1.x - p2.x) as f64 + (p1.y - p2.y) as f64 * (p1.y - p2.y) as f64,
    ) as i32
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
