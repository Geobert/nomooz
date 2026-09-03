use crate::{
    behavior::Behavior, bindings::Bindings, text_renderer::TextRenderer, theme::Theme,
    xkb_parser::XkbParser,
};

pub struct Dependencies {
    pub text_renderer: TextRenderer,
    pub xkb_parser: XkbParser,
    pub theme: Theme,
    pub bindings: Bindings,
    pub behavior: Behavior,
}
