use ori_native::prelude::*;

use crate::theme;

pub fn icon(bytes: &'static [u8]) -> Image {
    image(bytes).size(24.0, 24.0).tint(theme::TEXT)
}
