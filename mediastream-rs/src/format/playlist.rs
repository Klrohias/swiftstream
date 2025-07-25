use smol_str::SmolStr;
use std::collections::HashMap;

use crate::format::M3uMedia;

#[derive(Default)]
pub struct M3uPlaylist {
    pub title: Option<SmolStr>,
    pub attributes: HashMap<SmolStr, SmolStr>,
    pub medias: Vec<M3uMedia>,
}
