use smol_str::SmolStr;
use std::collections::HashMap;

use crate::format::M3uMedia;

#[derive(Default)]
pub struct M3uPlaylist {
    /// Title of this playlist
    pub title: Option<SmolStr>,
    /// Attributes of this playlist
    pub attributes: HashMap<SmolStr, SmolStr>,
    /// Medias of this playlist
    pub medias: Vec<M3uMedia>,
}
