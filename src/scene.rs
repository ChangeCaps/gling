use std::{
    ffi::OsStr,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    rc::Rc,
};

use ::gtk4::{
    gio::prelude::FileExt,
    prelude::{FileChooserExt, NativeDialogExt},
};
use kira::{
    Mix, Tween, Value,
    effect::reverb::{ReverbBuilder, ReverbHandle},
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    track::{TrackBuilder, TrackHandle},
};
use ori_native::prelude::*;
use uuid::Uuid;

use crate::{Audio, button, icon, label, slider, storage::v1, theme, uuid_map::UuidMap};

pub struct Scene {
    pub name: String,
    pub master: Master,
    pub sounds: UuidMap<Sound>,
}

pub struct Master {
    pub track: TrackHandle,
    pub gain: f32,
}

pub struct Sound {
    pub track: TrackHandle,
    pub name: String,
    pub gain: f32,

    pub reverb: Reverb,

    pub kind: v1::Kind,
    pub random: v1::Random,

    pub expand: bool,

    pub path: Option<PathBuf>,
    pub data: Option<StaticSoundData>,
    pub handle: Option<StaticSoundHandle>,
}

pub struct Reverb {
    pub handle: ReverbHandle,
    pub feedback: f32,
    pub damping: f32,
    pub width: f32,
    pub mix: f32,
}

impl Scene {
    pub fn new(audio: &mut Audio) -> eyre::Result<Self> {
        Self::from_data_v1(audio, Default::default())
    }

    pub fn from_data_v1(audio: &mut Audio, data: v1::Scene) -> eyre::Result<Self> {
        let mut master = Master::from_data(&mut audio.master, data.master)?;

        let sounds = data
            .sounds
            .into_iter()
            .map(|(id, data)| -> eyre::Result<_> {
                let sound = Sound::from_data_v1(&mut master.track, data)?;
                Ok((id, sound))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            name: data.name,
            master,
            sounds,
        })
    }

    pub fn to_data_v1(&self) -> v1::Scene {
        v1::Scene {
            name: self.name.clone(),
            master: self.master.to_data_v1(),
            sounds: self
                .sounds
                .iter()
                .map(|(id, data)| (id, data.to_data_v1()))
                .collect(),
        }
    }

    pub fn rewind(&mut self) {
        for track in self.sounds.values_mut_unordered() {
            if let Some(ref mut handle) = track.handle {
                handle.seek_to(0.0);
            }
        }
    }

    pub fn add_sound(&mut self) -> eyre::Result<Uuid> {
        let sound = Sound::new(&mut self.master.track)?;
        let uuid = self.sounds.add(sound);
        Ok(uuid)
    }

    pub fn start(&mut self) {
        for sound in self.sounds.values_mut_unordered() {
            sound.start(0.0);
        }
    }

    pub fn stop(&mut self) {
        for sound in self.sounds.values_mut_unordered() {
            sound.stop();
        }
    }

    pub fn tick(&mut self) {
        for sound in self.sounds.values_mut_unordered() {
            sound.tick();
        }
    }

    fn position(&self) -> f64 {
        self.sounds
            .values()
            .filter_map(|sound| sound.handle.as_ref())
            .map(|handle| handle.position())
            .next()
            .unwrap_or(0.0)
    }
}

impl Master {
    pub fn from_data(parent: &mut TrackHandle, data: v1::Master) -> eyre::Result<Self> {
        Ok(Self {
            track: parent.add_sub_track(Default::default())?,
            gain: data.gain,
        })
    }

    pub fn to_data_v1(&self) -> v1::Master {
        v1::Master { gain: self.gain }
    }
}

impl Sound {
    pub fn new(parent: &mut TrackHandle) -> eyre::Result<Self> {
        let mut sound = Self::from_data_v1(parent, Default::default())?;
        sound.expand = true;

        Ok(sound)
    }

    pub fn from_data_v1(parent: &mut TrackHandle, data: v1::Sound) -> eyre::Result<Self> {
        let mut builder = TrackBuilder::new();
        let reverb = Reverb::from_data_v1(&mut builder, data.reverb);

        Ok(Self {
            data: data
                .path
                .as_deref()
                .map(StaticSoundData::from_file)
                .transpose()?,

            track: parent.add_sub_track(builder)?,
            name: data.name,
            gain: data.gain,

            reverb,

            kind: data.kind,
            random: data.random,

            expand: false,

            path: data.path,

            handle: None,
        })
    }

    pub fn to_data_v1(&self) -> v1::Sound {
        v1::Sound {
            name: self.name.clone(),
            gain: self.gain,
            reverb: self.reverb.to_data_v1(),
            kind: self.kind,
            random: self.random.clone(),
            path: self.path.clone(),
        }
    }

    fn set_kind(&mut self, kind: v1::Kind, position: f64) {
        if !kind.is_looping() {
            self.stop();
        }

        self.kind = kind;
        self.start(position);
    }

    fn set_data(&mut self, data: StaticSoundData, position: f64) {
        self.data = Some(data);
        self.stop();
        self.start(position);
    }

    fn tick(&mut self) {
        if self.kind == v1::Kind::Random
            && rand::random::<f32>() < self.random.rate
            && let Some(ref data) = self.data
            && let Ok(mut handle) = self.track.play(data.clone())
        {
            if self.random.volume < 0.0 {
                let volume = rand::random_range(self.random.volume..=0.0);
                handle.set_volume(volume, Default::default());
            }

            if self.random.panning > 0.0 {
                let panning = rand::random_range(-self.random.panning..=self.random.panning);
                handle.set_panning(panning, Default::default());
            }

            self.handle = Some(handle);
        }
    }

    fn start(&mut self, position: f64) {
        if !self.kind.is_looping() {
            return;
        }

        if let Some(ref mut handle) = self.handle {
            if (handle.position() - position).abs() > 0.1 {
                handle.seek_to(position);
            }
        } else if let Some(ref data) = self.data
            && let Ok(mut handle) = self.track.play(data.clone())
        {
            handle.set_loop_region(..);
            handle.seek_to(position);
            self.handle = Some(handle);
        }
    }

    fn stop(&mut self) {
        if let Some(mut handle) = self.handle.take() {
            handle.stop(Default::default());
        }
    }
}

impl Reverb {
    pub fn from_data_v1(builder: &mut TrackBuilder, data: v1::Reverb) -> Self {
        let handle = builder.add_effect(ReverbBuilder {
            feedback: Value::Fixed(data.feedback as f64),
            damping: Value::Fixed(data.damping as f64),
            stereo_width: Value::Fixed(data.width as f64),
            mix: Value::Fixed(Mix(data.mix)),
        });

        Self {
            handle,
            feedback: data.feedback,
            damping: data.damping,
            width: data.width,
            mix: data.mix,
        }
    }

    pub fn to_data_v1(&self) -> v1::Reverb {
        v1::Reverb {
            feedback: self.feedback,
            damping: self.damping,
            width: self.width,
            mix: self.mix,
        }
    }
}

pub fn scene(scene: &Scene) -> impl View<Scene> + Layout + use<> {
    let name = row(textinput()
        .text(&scene.name)
        .placeholder("...")
        .size(20.0)
        .color(theme::TEXT)
        .family(theme::FONT)
        .newline(Newline::Never)
        .align(TextAlign::Center)
        .width(400.0)
        .accept_tab(false)
        .on_change(|scene: &mut Scene, name| scene.name = name))
    .border_bottom(1.0, theme::OUTLINE)
    .margin(20.0);

    column((name, column(()).flex(1.0), mixer(scene)))
        .align_items(Align::Center)
        .gap(40.0)
        .padding(20.0)
        .flex(1.0)
}

pub fn right_bar(scene: &Scene) -> impl View<Scene> + use<> {
    sounds(scene)
}

fn sounds(scene: &Scene) -> impl View<Scene> + Layout + use<> {
    column(
        list(scene.sounds.len() + 1, |scene: &Scene, i| {
            if let Some(id) = scene.sounds.get_uuid(i) {
                any(sound(id, &scene.sounds[id]))
            } else {
                any(add_sound())
            }
        })
        .flex(1.0)
        .gap(8.0),
    )
    .min_height(0.0)
    .width(400.0)
    .padding(8.0)
    .shadow(0.0, 0.0, 12.0, Color::BLACK.fade(0.3))
}

fn add_sound() -> impl View<Scene> + use<> {
    row(button::button(icon::plus(), |scene: &mut Scene| {
        let _ = scene.add_sound();
    }))
    .justify_content(Justify::Center)
}

fn remove_sound(id: Uuid) -> impl View<Scene> + use<> {
    button::button(icon::trash().tint(theme::RED), move |scene: &mut Scene| {
        if let Some(mut sound) = scene.sounds.remove(id) {
            sound.stop();
        }
    })
    .padding(6.0)
}

fn sound(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let name = textinput()
        .text(&sound.name)
        .color(theme::TEXT)
        .family(theme::FONT)
        .size(12.0)
        .newline(Newline::Never)
        .accept_tab(false)
        .on_change(move |scene: &mut Scene, text| scene.sounds[id].name = text)
        .flex(1.0);

    let properties = match sound.kind {
        v1::Kind::Music => any(column(reverb_properties(id, sound))),
        v1::Kind::Ambient => any(column(reverb_properties(id, sound))),
        v1::Kind::Random => any(column((
            random_properties(id, sound),
            reverb_properties(id, sound),
        ))),
        v1::Kind::Trigger => any(column(reverb_properties(id, sound))),
    };

    let header = row((
        row((expand(id, sound), remove_sound(id))),
        row(name)
            .padding_bottom(-2.0)
            .border_bottom(1.0, theme::OUTLINE)
            .flex(1.0),
    ))
    .align_self(Align::Stretch)
    .align_items(Align::Center)
    .gap(12.0);

    let body = match sound.expand {
        true => Some((row(kind(id, sound)), properties, select_file(id, sound))),
        false => None,
    };

    column((header, body))
        .padding(6.0)
        .background(theme::SURFACE)
}

fn expand(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let icon = match sound.expand {
        true => icon::eye(),
        false => icon::eye_closed(),
    };

    button::button(
        icon.size(24.0, 24.0).tint(theme::TEXT),
        move |scene: &mut Scene| {
            scene.sounds[id].expand ^= true;
        },
    )
    .padding(6.0)
}

fn kind(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let button = |kind| {
        let color = match sound.kind == kind {
            true => theme::ACCENT,
            false => theme::TEXT.fade(0.5),
        };

        transition(color, Ease(0.1), move |_, color| {
            button::button(
                kind_icon(kind).size(24.0, 24.0).tint(color),
                move |scene: &mut Scene| {
                    let position = scene.position();
                    scene.sounds[id].set_kind(kind, position);
                },
            )
            .padding(6.0)
        })
    };

    row((
        button(v1::Kind::Trigger),
        button(v1::Kind::Random),
        button(v1::Kind::Ambient),
        button(v1::Kind::Music),
    ))
}

fn kind_icon(kind: v1::Kind) -> Image {
    match kind {
        v1::Kind::Trigger => icon::target(),
        v1::Kind::Random => icon::die(),
        v1::Kind::Ambient => icon::cactus(),
        v1::Kind::Music => icon::notes(),
    }
}

fn properties<T, V>(title: &str, content: V) -> impl View<T> + use<T, V>
where
    T: 'static,
    V: ViewSeq<T> + 'static,
{
    any(column((
        label(title).size(12.0),
        column(content).padding(4.0).flex(1.0),
    ))
    .padding(8.0))
}

fn random_properties(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    properties(
        "Random",
        (
            random_rate(id, sound),
            random_volume(id, sound),
            random_panning(id, sound),
        ),
    )
}

fn random_rate(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "rate",
        1.0 / sound.random.rate,
        0.1..=500.0,
        Space::Log,
        move |scene, value| {
            scene.sounds[id].random.rate = 1.0 / value;
        },
    )
}

fn random_volume(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "volume",
        sound.random.volume,
        -60.0..=0.0,
        Space::Linear,
        move |scene, value| {
            scene.sounds[id].random.volume = value;
        },
    )
}

fn random_panning(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "panning",
        sound.random.panning,
        0.0..=1.0,
        Space::Linear,
        move |scene, value| {
            scene.sounds[id].random.panning = value;
        },
    )
}

fn reverb_properties(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    properties(
        "Reverb",
        (
            reverb_feedback(id, sound),
            reverb_damping(id, sound),
            reverb_width(id, sound),
            reverb_mix(id, sound),
        ),
    )
}

fn reverb_feedback(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "feedback",
        sound.reverb.feedback,
        0.0..=1.0,
        Space::Square,
        move |scene, value| {
            scene.sounds[id].reverb.feedback = value;
            scene.sounds[id]
                .reverb
                .handle
                .set_feedback(value as f64, Default::default());
        },
    )
}

fn reverb_damping(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "damping",
        sound.reverb.damping,
        0.0..=1.0,
        Space::Linear,
        move |scene, value| {
            scene.sounds[id].reverb.damping = value;
            scene.sounds[id]
                .reverb
                .handle
                .set_damping(value as f64, Default::default());
        },
    )
}

fn reverb_width(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "width",
        sound.reverb.width,
        0.0..=1.0,
        Space::Linear,
        move |scene, value| {
            scene.sounds[id].reverb.width = value;
            scene.sounds[id]
                .reverb
                .handle
                .set_stereo_width(value as f64, Default::default());
        },
    )
}

fn reverb_mix(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    property(
        "mix",
        sound.reverb.mix,
        0.0..=1.0,
        Space::Linear,
        move |scene, value| {
            scene.sounds[id].reverb.mix = value;
            scene.sounds[id]
                .reverb
                .handle
                .set_mix(Mix(value), Default::default());
        },
    )
}

fn property<F>(
    name: &'static str,
    value: f32,
    range: RangeInclusive<f32>,
    space: Space,
    on_input: F,
) -> impl View<Scene> + use<F>
where
    F: Fn(&mut Scene, f32) + 'static,
{
    let on_input = Rc::new(on_input);

    memo(value, move |_| {
        row((
            label(name).size(10.0).flex(3.0).flex_basis(0.0),
            numberinput(&format!("{value:.1}"), {
                let range = range.clone();
                let on_input = on_input.clone();

                move |scene, value| {
                    on_input(scene, value.clamp(*range.start(), *range.end()));
                }
            })
            .flex(2.0)
            .flex_basis(0.0),
            slider::slider()
                .value({
                    let value = space.map(value);
                    let start = space.map(*range.start());
                    let end = space.map(*range.end());

                    (value - start) / (end - start)
                })
                .direction(Direction::Horizontal)
                .knob_radius(8.0)
                .track_length(80.0)
                .padding(8.0)
                .flex(7.0)
                .flex_basis(0.0)
                .on_input(move |scene, value| {
                    let start = space.map(*range.start());
                    let end = space.map(*range.end());

                    let value = value * (end - start) + start;
                    let value = space.inverse(value);
                    on_input(scene, value);
                }),
        ))
        .align_items(Align::Center)
    })
}

enum Space {
    Linear,
    Square,
    Log,
}

impl Space {
    fn map(&self, x: f32) -> f32 {
        match self {
            Space::Linear => x,
            Space::Square => x * x,
            Space::Log => x.ln(),
        }
    }

    fn inverse(&self, x: f32) -> f32 {
        match self {
            Space::Linear => x,
            Space::Square => x.sqrt(),
            Space::Log => x.exp(),
        }
    }
}

fn select_file(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let view_id = ViewId::next();

    let path = sound
        .path
        .as_deref()
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .unwrap_or("");

    effect(
        button::button(
            row((
                icon::folder().size(20.0, 20.0),
                row(label(path).size(10.0))
                    .border_bottom(1.0, theme::OUTLINE)
                    .overflow(Overflow::Hidden)
                    .flex(1.0),
            ))
            .align_items(Align::Center)
            .gap(4.0)
            .flex(1.0),
            move |_| {
                Action::task(move |proxy| {
                    let chooser = ::gtk4::FileChooserNative::builder()
                        .title("Select audio file")
                        .build();

                    chooser.connect_response(move |dialog, response| {
                        if let ::gtk4::ResponseType::Accept = response {
                            let file = dialog.file();
                            proxy.message(Message::new(file, view_id));
                        }
                    });

                    chooser.show();

                    async {}
                })
            },
        ),
        receive(
            view_id,
            move |scene: &mut Scene, file: Option<::gtk4::gio::File>| {
                let position = scene.position();
                let sound = &mut scene.sounds[id];

                if let Some(file) = file
                    && let Some(path) = file.path()
                    && let Ok(data) = StaticSoundData::from_file(&path)
                {
                    sound.path = Some(path);
                    sound.set_data(data, position);
                }
            },
        ),
    )
}

fn mixer(scene: &Scene) -> impl View<Scene> + use<> {
    let sounds = scene.sounds.uuids().map(|uuid| {
        let track = map(
            mixer_track(&scene.sounds[uuid]),
            move |scene: &mut Scene, map| map(&mut scene.sounds[uuid]),
        );

        (uuid, track)
    });

    section(
        "Mixer",
        row((
            map(mixer_master(&scene.master), |scene: &mut Scene, map| {
                map(&mut scene.master)
            }),
            keyed(sounds),
        ))
        .gap(8.0),
    )
}

fn mixer_master(master: &Master) -> impl View<Master> + use<> {
    row(map_with(
        gain_slider("master", master.gain),
        |master: &mut Master, map| map(&mut master.gain, &mut master.track),
    ))
}

fn mixer_track(sound: &Sound) -> impl View<Sound> + use<> {
    row(map_with(
        gain_slider(&sound.name, sound.gain),
        |sound: &mut Sound, map| map(&mut sound.gain, &mut sound.track),
    ))
}

fn section<T>(name: &str, content: impl View<T>) -> impl View<T> + Layout {
    column((label(name), content))
        .min_width(0.0)
        .align_items(Align::Center)
        .gap(12.0)
}

fn gain_slider(name: &str, gain: f32) -> impl View<(f32, TrackHandle)> + use<> {
    const MIN: f32 = -60.0;
    const MAX: f32 = 12.0;
    const RANGE: f32 = MAX - MIN;

    let text = if gain == MIN {
        String::from("-inf")
    } else {
        format!("{:.1}", gain)
    };

    let name = name.to_string();

    memo(gain, move |_| {
        column((
            label(name)
                .size(12.0)
                .wrap(TextWrap::Word)
                .align(TextAlign::Center),
            slider::slider()
                .value((gain - MIN) / RANGE)
                .direction(Direction::Vertical)
                .track_length(200.0)
                .on_input(|(gain, track): &mut (f32, TrackHandle), value| {
                    *gain = value * RANGE + MIN;
                    track.set_volume(*gain, Tween::default());
                })
                .flex(1.0),
            column((
                numberinput(&text, |(gain, track): &mut (f32, TrackHandle), value| {
                    *gain = value.clamp(MIN, MAX);
                    track.set_volume(*gain, Tween::default());
                })
                .align(TextAlign::Center)
                .align_self(Align::Stretch),
                label("dB").size(10.0),
            ))
            .align_items(Align::Center)
            .align_self(Align::Stretch),
        ))
        .justify_content(Justify::Stretch)
        .align_items(Align::Center)
        .width(60.0)
    })
}

fn numberinput<T, A>(x: &str, mut on_input: impl FnMut(&mut T, f32) -> A + 'static) -> TextInput<T>
where
    A: Into<Action>,
{
    let x = x.to_string();

    textinput()
        .text(x.clone())
        .color(theme::TEXT)
        .family(theme::FONT)
        .size(10.0)
        .on_edited(move |data, value| {
            if x != value
                && let Ok(value) = value.parse::<f32>()
            {
                on_input(data, value).into()
            } else {
                Action::new()
            }
        })
        .newline(Newline::Never)
        .accept_tab(false)
}
