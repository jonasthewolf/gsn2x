#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SingleEdge {
    InContextOf,
    SupportedBy,
    Composite,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EdgeType {
    OneWay(SingleEdge),
    TwoWay((SingleEdge, SingleEdge)),
    Invisible,
}
