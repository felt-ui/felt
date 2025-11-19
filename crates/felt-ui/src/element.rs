use crate::Widget;

/// An element is a declarative description of a widget.
/// It is a builder that can construct a concrete Widget.
pub trait Element {
    fn build(self: Box<Self>) -> Box<dyn Widget>;
}

/// A trait for types that can be converted into an Element.
pub trait IntoElement {
    fn into_element(self) -> Box<dyn Element>;
}

impl<T: Element + 'static> IntoElement for T {
    fn into_element(self) -> Box<dyn Element> {
        Box::new(self)
    }
}
