#[derive(Debug, PartialEq)]
pub enum EdgeType {
    NoneToInContextOf,
    NoneToSupportedBy,
    NoneToComposite,
    InContextOfToSupportedBy,
    InContextOfToInContextOf,
    InContextOfToComposite,
    SupportedByToInContextOf,
    SupportedByToSupportedBy,
    SupportedByToComposite,
    CompositeToInContextOf,
    CompositeToSupportedBy,
    CompositeToComposite,
    Invisible,
}
