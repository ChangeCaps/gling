use ori_native::prelude::*;

use crate::theme;

pub fn icon(bytes: &'static [u8]) -> Image {
    image(bytes).size(24.0, 24.0).tint(theme::TEXT)
}

pub fn cactus() -> Image {
    icon(include_bytes!("icon/cactus.svg"))
}

pub fn die() -> Image {
    icon(include_bytes!("icon/die.svg"))
}

pub fn eye_closed() -> Image {
    icon(include_bytes!("icon/eye-closed.svg"))
}

pub fn eye() -> Image {
    icon(include_bytes!("icon/eye.svg"))
}

pub fn folder() -> Image {
    icon(include_bytes!("icon/folder.svg"))
}

pub fn notes() -> Image {
    icon(include_bytes!("icon/notes.svg"))
}

pub fn pause() -> Image {
    icon(include_bytes!("icon/pause.svg"))
}

pub fn play() -> Image {
    icon(include_bytes!("icon/play.svg"))
}

pub fn plus() -> Image {
    icon(include_bytes!("icon/plus.svg"))
}

pub fn rewind() -> Image {
    icon(include_bytes!("icon/rewind.svg"))
}

pub fn target() -> Image {
    icon(include_bytes!("icon/target.svg"))
}

pub fn trash() -> Image {
    icon(include_bytes!("icon/trash.svg"))
}
