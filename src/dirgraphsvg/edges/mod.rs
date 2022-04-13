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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cloning() {
        let e = EdgeType::Invisible;
        assert_eq!(e.clone(), e);
        let si = SingleEdge::Composite;
        assert_eq!(si.clone(), si);
    }

    #[test]
    fn formatting() {
        assert_eq!(
            format!("{:?}", EdgeType::OneWay(SingleEdge::InContextOf)),
            "OneWay(InContextOf)"
        );
    }
}
