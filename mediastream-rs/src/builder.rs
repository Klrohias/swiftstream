use crate::format::{M3uMedia, M3uPlaylist, directives};

impl ToString for M3uPlaylist {
    fn to_string(&self) -> String {
        let mut result = "#EXTM3U".to_string();
        // header
        let header = self
            .attributes
            .iter()
            .map(|(key, value)| format!("{}=\"{}\"", key, value))
            .collect::<Vec<_>>()
            .join(" ");
        result.push_str(&header);
        result.push('\n');

        // title
        if self.title.is_some() {
            let line = format!("#PLAYLIST:{}\n", self.title.as_ref().unwrap());
            result.push_str(&line);
        }

        // medias
        for it in self.medias.iter().map(|x| x.to_string()) {
            result.push_str(&it);
        }

        result
    }
}

impl ToString for M3uMedia {
    fn to_string(&self) -> String {
        let mut result = String::new();

        // extension data
        for it in self.extension_data.iter().map(|(key, value)| {
            if value.is_none() {
                key.to_string()
            } else {
                format!("{}:{}\n", key, value.as_ref().unwrap())
            }
        }) {
            result.push_str(&it);
        }

        // #EXTINF:duration attributes...,name
        let info = format!("{}:{}", directives::EXTINF, self.duration);
        result.push_str(&info);

        if !self.attributes.is_empty() {
            result.push(' ');
            let attribute_str = self
                .attributes
                .iter()
                .map(|(key, value)| format!("{}=\"{}\"", key, value))
                .collect::<Vec<_>>()
                .join(" ");
            result.push_str(&attribute_str);
        }

        result.push(',');
        if self.name.is_some() {
            result.push_str(self.name.as_ref().unwrap());
        }
        result.push('\n');

        // location
        result.push_str(&self.location);
        result.push('\n');

        result
    }
}
