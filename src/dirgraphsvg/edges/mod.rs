use std::ops::BitOr;

use crate::gsn::GsnEdgeType;

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
