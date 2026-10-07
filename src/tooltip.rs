use std::time::{Duration, Instant};

use ori_native::prelude::*;

use crate::{label, theme};

#[builder]
pub fn tooltip<T>(
    text: &'static str,
    contents: impl View<T>,
    #[default = Duration::from_millis(500)] delay: Duration,
) -> impl View<T> {
    struct State {
        view_id: ViewId,
        last_hover: Option<Instant>,
        show_tooltip: bool,
    }

    let mut contents = Some(contents);

    with(
        |_| State {
            view_id: ViewId::next(),
            last_hover: None,
            show_tooltip: false,
        },
        move |state, _| {
            let mut contents = contents.take();

            effect(
                pressable(move |(state, _): &(State, _), _| {
                    popup(
                        without(maybe(contents.take())),
                        state.show_tooltip.then(|| {
                            row(label(text).size(10.0).wrap(TextWrap::Word).flex(1.0))
                                .background(theme::BACKGROUND)
                                .corner(6.0)
                                .max_width(400.0)
                                .shadow(8.0, 8.0, 8.0, Color::BLACK.fade(0.3))
                                .margin(16.0)
                                .margin_top(8.0)
                                .padding(4.0)
                        }),
                    )
                })
                .on_hover(move |(state, _), hovered| {
                    if hovered {
                        state.last_hover = Some(Instant::now());
                        let view_id = state.view_id;

                        Action::spawn(async move {
                            tokio::time::sleep(delay).await;
                            Action::message((), view_id)
                        })
                    } else {
                        state.last_hover = None;
                        state.show_tooltip = false;
                        Action::new()
                    }
                }),
                receive(state.view_id, move |(state, _): &mut (State, _), _: ()| {
                    if let Some(last_hover) = state.last_hover
                        && last_hover.elapsed() >= delay
                    {
                        state.show_tooltip = true;
                    }
                }),
            )
        },
    )
}
