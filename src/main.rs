use std::{fs, path::PathBuf, time::Duration};

use kira::{AudioManager, Tween, track::TrackHandle};
use ori_native::prelude::*;
use uuid::Uuid;

use crate::{scene::Scene, uuid_map::UuidMap};

mod button;
mod icon;
mod scene;
mod slider;
mod storage;
mod theme;
mod tooltip;
mod uuid_map;

fn main() -> eyre::Result<()> {
    App::init_log();

    let mut data = Data::load()?;

    App::new().run(&mut data, ui)?;

    data.store()?;

    Ok(())
}

struct Data {
    audio: Audio,
    scenes: UuidMap<Scene>,
    selected_scene: Option<Uuid>,
    is_playing: bool,
}

struct Audio {
    manager: AudioManager,
    master: TrackHandle,
}

impl Audio {
    fn new() -> eyre::Result<Self> {
        let mut manager = AudioManager::new(Default::default())?;
        let master = manager.add_sub_track(Default::default())?;

        Ok(Self { manager, master })
    }
}

impl Data {
    fn add_scene(&mut self) -> eyre::Result<Uuid> {
        let scene = Scene::new(&mut self.audio)?;
        let id = self.scenes.add(scene);
        self.selected_scene = Some(id);
        Ok(id)
    }

    fn select_scene(&mut self, id: Uuid) {
        if let Some(scene) = self.selected_scene.take() {
            self.scenes[scene].stop();
        }

        self.selected_scene = Some(id);
        self.scenes[id].start();
    }

    fn tick(&mut self) {
        if let Some(id) = self.selected_scene
            && self.is_playing
        {
            self.scenes[id].tick();
        }
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("storage.ron")
    }

    fn load() -> eyre::Result<Self> {
        let data = Self::load_data().unwrap_or_default();

        match data {
            storage::Data::V1(data) => Self::from_data_v1(data),
        }
    }

    fn load_data() -> eyre::Result<storage::Data> {
        let ron = fs::read_to_string(Self::storage_path())?;
        ron::from_str(&ron).map_err(Into::into)
    }

    fn from_data_v1(data: storage::v1::Data) -> eyre::Result<Self> {
        let mut audio = Audio::new()?;

        let scenes = data
            .scenes
            .into_iter()
            .map(|(id, scene)| -> eyre::Result<_> {
                Ok((id, Scene::from_data_v1(&mut audio, scene)?))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            audio,
            scenes,
            selected_scene: None,
            is_playing: true,
        })
    }

    fn to_data_v1(&self) -> storage::v1::Data {
        storage::v1::Data {
            scenes: self
                .scenes
                .iter()
                .map(|(id, scene)| (id, scene.to_data_v1()))
                .collect(),
        }
    }

    fn store(&self) -> eyre::Result<()> {
        let data = storage::Data::V1(self.to_data_v1());
        let ron = ron::to_string(&data)?;
        fs::write(Self::storage_path(), ron).map_err(Into::into)
    }
}

fn label(label: impl Into<String>) -> Text {
    text(label).color(theme::TEXT).family(theme::FONT)
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    effect(
        window(
            column((
                row((left_bar(data), selected_scene(data), right_bar(data)))
                    .flex(1.0)
                    .min_height(0.0),
                top_bar(data),
            ))
            .min_height(0.0)
            .reverse(true)
            .flex(1.0)
            .background(theme::BACKGROUND),
        ),
        task(
            |_, sink| async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    sink.send(());
                }
            },
            |data: &mut Data, _, _| {
                data.tick();
                Action::new()
            },
        ),
    )
}

fn selected_scene(data: &Data) -> impl View<Data> + use<> {
    match data.selected_scene {
        Some(uuid) => any(map(
            scene::scene(&data.scenes[uuid]),
            move |data: &mut Data, map| map(&mut data.scenes[uuid]),
        )),
        None => any(row(()).flex(1.0)),
    }
}

fn right_bar(data: &Data) -> Option<impl View<Data> + use<>> {
    data.selected_scene.map(|id| {
        map(
            scene::right_bar(&data.scenes[id]),
            move |data: &mut Data, map| map(&mut data.scenes[id]),
        )
    })
}

fn top_bar(data: &Data) -> impl View<Data> + use<> {
    row(pause_play_rewind(data))
        .justify_content(Justify::Center)
        .align_items(Align::Center)
        .align_self(Align::Stretch)
        .padding(20.0)
        .min_width(500.0)
        .background(theme::BACKGROUND)
        .shadow(0.0, 0.0, 12.0, Color::BLACK.fade(0.3))
}

fn left_bar(data: &Data) -> impl View<Data> + use<> {
    column((
        list(data.scenes.len(), |data: &Data, i| {
            let id = data.scenes.get_uuid(i).unwrap();
            scene_button(data, &data.scenes[i], id)
        })
        .align_self(Align::Stretch)
        .gap(4.0),
        add_scene(),
    ))
    .align_items(Align::Center)
    .padding(20.0)
    .gap(12.0)
    .width(400.0)
    .shadow(0.0, 0.0, 12.0, Color::BLACK.fade(0.3))
    .background(theme::BACKGROUND)
}

fn scene_button(data: &Data, scene: &Scene, id: Uuid) -> impl View<Data> + use<> {
    let color = if data.selected_scene == Some(id) {
        theme::ACCENT
    } else {
        theme::TEXT.fade(0.5)
    };

    let name = scene.name.clone();

    transition(color, Ease(0.1), move |_, color| {
        button::button(label(&name).color(color), move |data: &mut Data| {
            data.select_scene(id);
        })
    })
}

fn add_scene() -> impl View<Data> + use<> {
    button::button(icon::plus(), |data: &mut Data| {
        let _ = data.add_scene();
    })
}

fn pause_play_rewind(data: &Data) -> impl View<Data> + use<> {
    row((rewind(), pause(data), play(data)))
}

fn rewind() -> impl View<Data> {
    button::button(icon::rewind(), |data: &mut Data| {
        if let Some(id) = data.selected_scene {
            data.scenes[id].rewind();
        }
    })
}

fn pause(data: &Data) -> impl View<Data> + use<> {
    let color = match data.is_playing {
        true => theme::TEXT,
        false => theme::ACCENT,
    };

    transition(color, Ease(0.05), |_, color| {
        button::button(icon::pause().tint(color), |data: &mut Data| {
            data.audio.master.pause(Tween::default());
            data.is_playing = false;
        })
    })
}

fn play(data: &Data) -> impl View<Data> + use<> {
    let color = match data.is_playing {
        true => theme::ACCENT,
        false => theme::TEXT,
    };

    transition(color, Ease(0.05), |_, color| {
        button::button(icon::play().tint(color), |data: &mut Data| {
            data.audio.master.resume(Tween::default());
            data.is_playing = true;
        })
    })
}
