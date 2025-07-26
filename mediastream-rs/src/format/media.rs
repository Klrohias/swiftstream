use std::collections::HashMap;

use smol_str::SmolStr;

pub struct M3uMedia {
    pub name: Option<SmolStr>,
    pub duration: f32,
    pub location: SmolStr,
    pub attributes: HashMap<SmolStr, SmolStr>,
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
