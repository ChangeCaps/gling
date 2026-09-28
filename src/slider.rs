use ori_native::{PressableEvent, prelude::*};

use crate::theme;

pub fn slider<T>() -> Slider<T> {
    Slider::new()
}

pub struct Slider<T> {
    value: Option<f32>,

    track_length: Length,
    track_width: f32,
    knob_radius: f32,
    knob_width: f32,
    padding: f32,
    flex: f32,
    flex_basis: Option<Length>,
    direction: Direction,

    #[allow(clippy::type_complexity)]
    on_input: Box<dyn FnMut(&mut T, f32) -> Action>,
}

impl<T> Slider<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            track_length: Length::Fract(1.0),
            track_width: theme::slider::TRACK_WIDTH,
            knob_radius: theme::slider::KNOB_RADIUS,
            knob_width: theme::slider::KNOB_WIDTH,
            padding: theme::slider::PADDING,
            direction: Direction::Horizontal,
            flex: 0.0,
            flex_basis: None,
            on_input: Box::new(|_, _| Action::new()),
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = Some(value);
        self
    }

    pub fn track_length(mut self, track_length: impl Into<Length>) -> Self {
        self.track_length = track_length.into();
        self
    }

    pub fn knob_radius(mut self, radius: f32) -> Self {
        self.knob_radius = radius;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn flex(mut self, flex: f32) -> Self {
        self.flex = flex;
        self
    }

    pub fn flex_basis(mut self, basis: impl Into<Length>) -> Self {
        self.flex_basis = Some(basis.into());
        self
    }

    pub fn on_input<A>(mut self, mut on_input: impl FnMut(&mut T, f32) -> A + 'static) -> Self
    where
        A: Into<Action>,
    {
        self.on_input = Box::new(move |data, progress| on_input(data, progress).into());
        self
    }
}

impl<T> BuildMarker for Slider<T> {}
impl<T> BuildView<Context, T> for Slider<T>
where
    T: 'static,
{
    fn build(mut self) -> BoxedView<T> {
        #[derive(Default)]
        struct State {
            drag: Option<f32>,
            progress: f32,
            track_length: f32,
        }

        any(with(
            move |_| State {
                drag: None,
                progress: self.value.unwrap_or(0.0),
                track_length: 0.0,
            },
            move |_, _| {
                effect(
                    pressable(move |(state, _): &(State, _), press| {
                        let inner = match self.direction {
                            Direction::Horizontal => row(())
                                .background(theme::OUTLINE)
                                .height(Fract(1.0))
                                .width(Fract(state.progress)),
                            Direction::Vertical => column(())
                                .background(theme::OUTLINE)
                                .width(Fract(1.0))
                                .height(Fract(state.progress)),
                        };

                        let (width, height) = match self.direction {
                            Direction::Horizontal => (self.track_length, self.track_width.into()),
                            Direction::Vertical => (self.track_width.into(), self.track_length),
                        };

                        let justify = match self.direction {
                            Direction::Horizontal => Justify::Start,
                            Direction::Vertical => Justify::End,
                        };

                        let track = on_layout(
                            flex(inner)
                                .direction(self.direction)
                                .width(width)
                                .height(height)
                                .corner(self.track_width / 2.0)
                                .flex(self.flex)
                                .justify_content(justify)
                                .overflow(Overflow::Hidden)
                                .background(theme::OUTLINE.lighten(0.5)),
                            move |(state, _): &mut (State, _), width, height| {
                                state.track_length = match self.direction {
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
                            .size(self.knob_radius * 2.0, self.knob_radius * 2.0)
                            .corner(self.knob_radius)
                            .border(self.knob_width, color)
                            .position(Position::Absolute);

                        let inset =
                            state.progress * state.track_length + self.padding - self.knob_radius;

                        let knob = match self.direction {
                            Direction::Horizontal => knob.left(inset),
                            Direction::Vertical => knob.bottom(inset),
                        };

                        let mut view = flex((track, knob))
                            .direction(self.direction)
                            .align_items(Align::Center)
                            .padding(self.padding)
                            .flex(self.flex);

                        view.get_layout_style_mut().flex_basis = self.flex_basis;

                        view
                    })
                    .on_event(move |(state, data), event| match event {
                        PressableEvent::Pressed(event) => {
                            let position = match self.direction {
                                Direction::Horizontal => event.position.x,
                                Direction::Vertical => event.position.y,
                            };

                            let progress = match self.direction {
                                Direction::Horizontal => state.progress,
                                Direction::Vertical => 1.0 - state.progress,
                            };

                            let track_position = position - self.padding;
                            let knob_position = progress * state.track_length;
                            let knob_start = knob_position - self.knob_radius;
                            let knob_end = knob_position + self.knob_radius;

                            if track_position >= knob_start && track_position <= knob_end {
                                state.drag = Some(knob_position - track_position);

                                Action::new()
                            } else {
                                let progress =
                                    (track_position / state.track_length).clamp(0.0, 1.0);

                                state.progress = match self.direction {
                                    Direction::Horizontal => progress,
                                    Direction::Vertical => 1.0 - progress,
                                };

                                state.drag = Some(0.0);

                                (self.on_input)(data, state.progress).with_rebuild(true)
                            }
                        }

                        PressableEvent::Moved(event) => {
                            if let Some(offset) = state.drag {
                                let position = match self.direction {
                                    Direction::Horizontal => event.position.x,
                                    Direction::Vertical => event.position.y,
                                };

                                let track_position = position - self.padding + offset;
                                let progress =
                                    (track_position / state.track_length).clamp(0.0, 1.0);

                                state.progress = match self.direction {
                                    Direction::Horizontal => progress,
                                    Direction::Vertical => 1.0 - progress,
                                };

                                (self.on_input)(data, state.progress).with_rebuild(true)
                            } else {
                                Action::new()
                            }
                        }

                        PressableEvent::Released(..) | PressableEvent::Cancelled(..) => {
                            state.drag = None;
                            Action::new()
                        }

                        _ => Action::new(),
                    }),
                    mutate(move |(state, _): &mut (State, _)| {
                        if let Some(value) = self.value {
                            state.progress = value;
                        }
                    }),
                )
            },
        ))
    }
}
