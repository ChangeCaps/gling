use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use ::gtk4::{
    gio::prelude::FileExt,
    prelude::{FileChooserExt, NativeDialogExt},
};
use kira::{
    Tween,
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    track::TrackHandle,
};
use ori_native::prelude::*;
use serde::{Deserialize, Serialize};
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

    pub kind: Kind,
    pub rate: f32,

    pub path: Option<PathBuf>,
    pub data: Option<StaticSoundData>,
    pub handle: Option<StaticSoundHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Music,
    Ambient,
    Random,
    Trigger,
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
        Self::from_data_v1(parent, Default::default())
    }

    pub fn from_data_v1(parent: &mut TrackHandle, data: v1::Sound) -> eyre::Result<Self> {
        Ok(Self {
            data: data
                .path
                .as_deref()
                .map(StaticSoundData::from_file)
                .transpose()?,

            track: parent.add_sub_track(Default::default())?,
            name: data.name,
            gain: data.gain,

            kind: data.kind,
            rate: data.rate,

            path: data.path,

            handle: None,
        })
    }

    pub fn to_data_v1(&self) -> v1::Sound {
        v1::Sound {
            name: self.name.clone(),
            gain: self.gain,
            kind: self.kind,
            rate: self.rate,
            path: self.path.clone(),
        }
    }

    fn set_kind(&mut self, kind: Kind, position: f64) {
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
        if self.kind == Kind::Random
            && rand::random::<f32>() < self.rate
            && let Some(ref data) = self.data
            && let Ok(handle) = self.track.play(data.clone())
        {
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

impl Kind {
    pub fn is_looping(&self) -> bool {
        matches!(self, Self::Music | Self::Ambient)
    }
}

pub fn scene(scene: &Scene) -> impl View<Scene> + use<> {
    let name = row(textinput()
        .text(&scene.name)
        .placeholder("...")
        .size(20.0)
        .color(theme::TEXT)
        .family(theme::FONT)
        .newline(Newline::None)
        .align(TextAlign::Center)
        .width(200.0)
        .accept_tab(false)
        .on_change(|scene: &mut Scene, name| scene.name = name))
    .border_bottom(1.0, theme::OUTLINE);

    column((name, sounds(scene), mixer(scene)))
        .gap(40.0)
        .padding(20.0)
        .min_width(0.0)
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .flex(1.0)
}

fn sounds(scene: &Scene) -> impl View<Scene> + Layout + use<> {
    let sounds = scene.sounds.iter().map(|(id, sound)| {
        let sound = self::sound(id, sound);
        (id, sound)
    });

    section(
        "Sounds",
        hscroll(row((keyed(sounds), add_sound())).align_items(Align::Center)).max_width(Fract(1.0)),
    )
    .max_width(Fract(1.0))
}

fn add_sound() -> impl View<Scene> + use<> {
    button::button(
        icon::icon(include_bytes!("icon/plus.svg")),
        |scene: &mut Scene| {
            let _ = scene.add_sound();
        },
    )
}

fn remove_sound(id: Uuid) -> impl View<Scene> + use<> {
    button::button(
        icon::icon(include_bytes!("icon/trash.svg")).tint(theme::RED),
        move |scene: &mut Scene| {
            if let Some(mut sound) = scene.sounds.remove(id) {
                sound.stop();
            }
        },
    )
    .padding(6.0)
}

fn sound(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let name = textinput()
        .text(&sound.name)
        .color(theme::TEXT)
        .family(theme::FONT)
        .size(12.0)
        .newline(Newline::None)
        .accept_tab(false)
        .on_change(move |scene: &mut Scene, text| scene.sounds[id].name = text)
        .flex(1.0);

    let properties = match sound.kind {
        Kind::Music => properties(()),
        Kind::Ambient => properties(()),
        Kind::Random => properties(probability(id, sound)),
        Kind::Trigger => properties(()),
    };

    column((
        column((
            row((
                row(name).border_bottom(1.0, theme::OUTLINE).flex(1.0),
                remove_sound(id),
            ))
            .align_self(Align::Stretch)
            .align_items(Align::Center)
            .gap(6.0),
            row(kind(id, sound)),
        ))
        .align_self(Align::Stretch)
        .align_items(Align::Center),
        properties,
        select_file(id, sound),
    ))
    .height(300.0)
    .margin(10.0)
    .padding(16.0)
    .corner(12.0)
    .justify_content(Justify::SpaceBetween)
    .shadow(0.0, 0.0, 12.0, Color::BLACK.fade(0.3))
}

fn kind(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let button = |kind, icon| {
        let color = match sound.kind == kind {
            true => theme::ACCENT,
            false => theme::TEXT.fade(0.5),
        };

        transition(color, Ease(0.1), move |_, color| {
            button::button(
                icon::icon(icon).size(24.0, 24.0).tint(color),
                move |scene: &mut Scene| {
                    let position = scene.position();
                    scene.sounds[id].set_kind(kind, position);
                },
            )
            .padding(6.0)
        })
    };

    row((
        button(Kind::Trigger, include_bytes!("icon/crosshair.svg")),
        button(Kind::Random, include_bytes!("icon/die.svg")),
        button(Kind::Ambient, include_bytes!("icon/cactus.svg")),
        button(Kind::Music, include_bytes!("icon/notes.svg")),
    ))
}

fn properties<T>(content: impl ViewSeq<T> + 'static) -> BoxedView<T>
where
    T: 'static,
{
    any(column(content).padding(4.0).flex(1.0))
}

fn probability(id: Uuid, sound: &Sound) -> impl View<Scene> + use<> {
    let probability = format!("{:.1}", 1.0 / sound.rate);

    row((
        label("rate: ").size(10.0),
        textinput()
            .text(probability)
            .size(10.0)
            .color(theme::TEXT)
            .family(theme::FONT)
            .newline(Newline::None)
            .accept_tab(false)
            .width(40.0)
            .on_submit(move |scene: &mut Scene, value| {
                if let Ok(value) = value.parse::<f32>() {
                    scene.sounds[id].rate = (1.0 / value).clamp(0.0, 1.0);
                }
            }),
    ))
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
                icon::icon(include_bytes!("icon/folder.svg")).size(20.0, 20.0),
                row(label(path).size(10.0))
                    .width(200.0)
                    .border_bottom(1.0, theme::OUTLINE)
                    .overflow(Overflow::Hidden),
            ))
            .align_items(Align::Center)
            .gap(4.0),
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

    column((
        label(name)
            .size(12.0)
            .wrap(TextWrap::Word)
            .align(TextAlign::Center),
        slider::slider()
            .value((gain - MIN) / RANGE)
            .direction(Direction::Vertical)
            .on_input(|(gain, track): &mut (f32, TrackHandle), value| {
                *gain = value * RANGE + MIN;
                track.set_volume(*gain, Tween::default());
            })
            .flex(1.0),
        column((
            textinput()
                .text(text)
                .color(theme::TEXT)
                .family(theme::FONT)
                .size(10.0)
                .on_submit(|(gain, track): &mut (f32, TrackHandle), value| {
                    if let Ok(value) = value.parse::<f32>() {
                        *gain = value.clamp(MIN, MAX);
                        track.set_volume(*gain, Tween::default());
                    }
                })
                .align(TextAlign::Center)
                .newline(Newline::None)
                .accept_tab(false)
                .align_self(Align::Stretch),
            label("dB").size(10.0),
        ))
        .align_items(Align::Center)
        .align_self(Align::Stretch),
    ))
    .justify_content(Justify::Stretch)
    .align_items(Align::Center)
    .width(60.0)
}
