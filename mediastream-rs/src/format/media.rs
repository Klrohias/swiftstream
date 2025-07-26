use std::collections::HashMap;

use smol_str::SmolStr;

pub struct M3uMedia {
    /// Name of this media
    pub name: Option<SmolStr>,
    /// How long do this media will last
    pub duration: f32,
    /// Location (relative or absolute URL) of this media
    pub location: SmolStr,
    /// Attributes of this media
    pub attributes: HashMap<SmolStr, SmolStr>,
    /// Directives that not been parsed
    pub extension_data: HashMap<SmolStr, Option<SmolStr>>,
}

impl Default for M3uMedia {
    fn default() -> Self {
        Self {
            name: None,
            duration: -1.0,
            attributes: HashMap::new(),
            extension_data: HashMap::new(),
            location: SmolStr::new(""),
        }
    }
}
