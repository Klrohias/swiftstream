use std::fmt::Display;

use crate::format::{M3uMedia, M3uPlaylist, directives};

impl Display for M3uPlaylist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // header
        write!(f, "{}", directives::EXTM3U)?;
        for (key, value) in self.attributes.iter() {
            write!(f, " {}=\"{}\"", key, value)?;
        }
        write!(f, "\n")?;

        // title
        if self.title.is_some() {
            writeln!(
                f,
                "{}:{}",
                directives::PLAYLIST,
                self.title.as_ref().unwrap()
            )?;
        }

        // medias
        for it in self.medias.iter() {
            write!(f, "\n")?;
            it.fmt(f)?;
        }

        Ok(())
    }
}

impl Display for M3uMedia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // extension data
        for (key, value) in self.extension_data.iter() {
            if value.is_none() {
                writeln!(f, "{}", key)?;
            } else {
                writeln!(f, "{}:{}", key, value.as_ref().unwrap())?;
            }
        }

        // #EXTINF:duration attributes...,name
        write!(f, "{}:{}", directives::EXTINF, self.duration)?;
        for (key, value) in self.attributes.iter() {
            write!(f, " {}=\"{}\"", key, value)?;
        }

        write!(f, ",")?;
        if self.name.is_some() {
            write!(f, "{}", self.name.as_ref().unwrap())?;
        }
        write!(f, "\n")?;

        writeln!(f, "{}", self.location)?;

        Ok(())
    }
}
