use crate::arena::ElementId;

#[derive(Debug)]
pub enum Mutation<'a> {
    SetAttribute {
        name: &'static str,
        value: &'a str,
        id: ElementId,
    },

    LoadTemplate {
        name: &'static str,
        id: ElementId,
    },

    HydrateText {
        path: &'static [u8],
        value: &'a str,
        id: ElementId,
    },

    SetText {
        value: &'a str,
        id: ElementId,
    },

    ReplacePlaceholder {
        path: &'static [u8],
        id: ElementId,
    },

    AssignId {
        path: &'static [u8],
        id: ElementId,
    },

    // Take the current element and replace it with the element with the given id.
    Replace {
        id: ElementId,
    },
}
