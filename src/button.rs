use ori_native::prelude::*;

use crate::tooltip;

#[builder]
pub fn button<T>(
    contents: impl View<T> + 'static,
    mut on_click: impl (FnMut(&mut T) -> impl Into<Action>) + 'static,

    #[default] tooltip: Option<&'static str>,

    #[layout] layout: LayoutStyle,
    #[default = Sides::from(8.0)]
    #[padding]
    padding: Sides<Length>,
) -> impl View<T>
where
    T: 'static,
{
    let mut contents = Some(contents);

    let view = pressable(move |_, state| {
        let color = if state.pressed {
            Color::BLACK.fade(0.1)
        } else if state.hovered {
            Color::BLACK.fade(0.05)
        } else {
            Color::TRANSPARENT
        };

        let mut contents = contents.take();

        transition(color, Ease(0.1), move |_, color| {
            row(maybe(contents.take()))
                .padding(padding)
                .corner(6.0)
                .background(color)
                .set_layout(layout)
        })
    })
    .on_press(move |data, _| (on_click)(data));

    match tooltip {
        Some(text) => any(tooltip::tooltip(text, view)),
        None => any(view),
    }
}
