use ori_native::{PressableEvent, prelude::*};

use crate::theme;

#[builder]
pub fn slider<T>(
    #[default] value: Option<f32>,
    #[default = Length::Fract(1.0)] track_length: Length,
    #[default = theme::slider::TRACK_WIDTH] track_width: f32,
    #[default = theme::slider::KNOB_RADIUS] knob_radius: f32,
    #[default = theme::slider::KNOB_WIDTH] knob_width: f32,
    #[default = theme::slider::PADDING] padding: f32,
    #[default = 0.0] flex: f32,
    #[default = None] flex_basis: Option<Length>,
    #[default = Direction::Horizontal] direction: Direction,
    #[default = false] scrollable: bool,

    #[default = |_, _| Action::new()] mut on_input: impl (FnMut(&mut T, f32) -> impl Into<Action>)
    + 'static,
) -> impl View<T> {
    #[allow(clippy::type_complexity)]
    struct State<T> {
        drag: Option<f32>,
        progress: f32,
        track_length: f32,
        on_input: Box<dyn FnMut(&mut T, f32) -> Action>,
    }

    with(
        move |_| State {
            drag: None,
            progress: value.unwrap_or(0.0),
            track_length: 0.0,
            on_input: Box::new(|_, _| Action::new()),
        },
        move |_, _| {
            pressable(move |(state, _): &(State<T>, _), press| {
                let inner = match direction {
                    Direction::Horizontal => row(())
                        .background(theme::OUTLINE)
                        .height(Fract(1.0))
                        .width(Fract(state.progress)),
                    Direction::Vertical => column(())
                        .background(theme::OUTLINE)
                        .width(Fract(1.0))
                        .height(Fract(state.progress)),
                };

                let (width, height) = match direction {
                    Direction::Horizontal => (track_length, track_width.into()),
                    Direction::Vertical => (track_width.into(), track_length),
                };

                let justify = match direction {
                    Direction::Horizontal => Justify::Start,
                    Direction::Vertical => Justify::End,
                };

                let track = on_layout(
                    row(inner)
                        .direction(direction)
                        .width(width)
                        .height(height)
                        .corner(track_width / 2.0)
                        .flex(flex)
                        .justify_content(justify)
                        .overflow(Overflow::Hidden)
                        .background(theme::OUTLINE.lighten(0.5)),
                    move |(state, _): &mut (State<T>, _), width, height| {
                        state.track_length = match direction {
                            Direction::Horizontal => width,
                            Direction::Vertical => height,
                        }
                    },
                );

                let color = if press.pressed {
                    theme::OUTLINE.lighten(0.4)
                } else {
                    theme::OUTLINE.lighten(0.5)
                };

                let knob = column(())
                    .size(knob_radius * 2.0, knob_radius * 2.0)
                    .corner(knob_radius)
                    .border(knob_width, color)
                    .position(Position::Absolute);

                let inset = state.progress * state.track_length + padding - knob_radius;

                let knob = match direction {
                    Direction::Horizontal => knob.left(inset),
                    Direction::Vertical => knob.bottom(inset),
                };

                let mut view = row((track, knob))
                    .direction(direction)
                    .align_items(Align::Center)
                    .padding(padding)
                    .flex(flex);

                view.get_layout_style_mut().flex_basis = flex_basis;

                view
            })
            .scrollable(scrollable)
            .on_event(move |(state, data), event| match event {
                PressableEvent::Pressed(event) => {
                    let position = match direction {
                        Direction::Horizontal => event.position.x,
                        Direction::Vertical => event.position.y,
                    };

                    let progress = match direction {
                        Direction::Horizontal => state.progress,
                        Direction::Vertical => 1.0 - state.progress,
                    };

                    let track_position = position - padding;
                    let knob_position = progress * state.track_length;
                    let knob_start = knob_position - knob_radius;
                    let knob_end = knob_position + knob_radius;

                    if track_position >= knob_start && track_position <= knob_end {
                        state.drag = Some(knob_position - track_position);

                        Action::new()
                    } else {
                        let progress = (track_position / state.track_length).clamp(0.0, 1.0);

                        state.progress = match direction {
                            Direction::Horizontal => progress,
                            Direction::Vertical => 1.0 - progress,
                        };

                        state.drag = Some(0.0);

                        (state.on_input)(data, state.progress).with_rebuild(true)
                    }
                }

                PressableEvent::Moved(event) => {
                    if let Some(offset) = state.drag {
                        let position = match direction {
                            Direction::Horizontal => event.position.x,
                            Direction::Vertical => event.position.y,
                        };

                        let track_position = position - padding + offset;
                        let progress = (track_position / state.track_length).clamp(0.0, 1.0);

                        state.progress = match direction {
                            Direction::Horizontal => progress,
                            Direction::Vertical => 1.0 - progress,
                        };

                        (state.on_input)(data, state.progress).with_rebuild(true)
                    } else {
                        Action::new()
                    }
                }

                PressableEvent::Released(..) | PressableEvent::Cancelled(..) => {
                    state.drag = None;
                    Action::new()
                }

                PressableEvent::Scrolled(event) => {
                    state.progress -= event.dy * 0.05;
                    state.progress = state.progress.clamp(0.0, 1.0);

                    (state.on_input)(data, state.progress).with_rebuild(true)
                }

                _ => Action::new(),
            })
        },
    )
    .update(move |state, _| {
        if let Some(value) = value {
            state.progress = value;
        }

        state.on_input = Box::new(move |data, value| on_input(data, value).into());
    })
}
