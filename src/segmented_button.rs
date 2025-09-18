use iced::{
    widget::{
        button::Style,
        button, Button
    }, Background, Element, Theme
};

pub fn segmented_button<'a, Message, Renderer, V, F>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    value: V,
    selected: Option<V>,
    f: F
) -> Button<'a, Message, Theme, Renderer>

where
    Renderer: iced_core::Renderer,
    V: Eq + Copy,
    F: FnOnce(V) -> Message
{
    let base = button(content).style(|theme, status| {
        let base_style = button::secondary(theme, button::Status::Active);
        let palette = theme.extended_palette();
        match status {
            button::Status::Active | button::Status::Pressed => Style { background: base_style.background.map(|background| background.scale_alpha(0.75)), text_color: base_style.text_color.scale_alpha(0.5), ..base_style },
            button::Status::Hovered => Style { background: Some(Background::Color(palette.secondary.base.color)), ..base_style },
            button::Status::Disabled => Style {background: Some(Background::Color(palette.secondary.strong.color)), ..base_style }
        }
    });
    if selected != Some(value) {
        base.on_press(f(value))
    }
     else {
        base
    }
}
