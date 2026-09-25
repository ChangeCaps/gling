use ori_native::{LayoutStyle, prelude::*};

pub fn button<T, V, A>(contents: V, mut on_click: impl FnMut(&mut T) -> A + 'static) -> Button<T, V>
where
    A: Into<Action>,
{
    Button {
        contents,
        on_click: Box::new(move |data| on_click(data).into()),
        layout: LayoutStyle::default(),
        padding: Sides::from(8.0),
    }
}

pub struct Button<T, V> {
    pub contents: V,
    pub on_click: Box<dyn FnMut(&mut T) -> Action>,
    pub layout: LayoutStyle,
    pub padding: Sides<Length>,
}

impl<T, V> Layout for Button<T, V> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

impl<T, V> Padding for Button<T, V> {
    fn get_padding_mut(&mut self) -> &mut Sides<Length> {
        &mut self.padding
    }
}

impl<T, V> BuildMarker for Button<T, V> {}
impl<T, V> BuildView<Context, T> for Button<T, V>
where
    T: 'static,
    V: View<T> + 'static,
{
    fn build(mut self) -> BoxedView<T> {
        let mut contents = Some(self.contents);
        any(pressable(move |_, state| {
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
                    .padding(self.padding)
                    .corner(6.0)
                    .background(color)
                    .layout(self.layout)
            })
        })
        .on_press(move |data, _| (self.on_click)(data)))
    }
}
