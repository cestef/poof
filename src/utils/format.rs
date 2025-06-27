use owo_colors::OwoColorize;

pub trait ReducedId {
    fn reduced(&self) -> String;
}

impl ReducedId for iroh::NodeId {
    fn reduced(&self) -> String {
        let id_str = self.to_string();
        format!(
            "{}{}{}",
            (&id_str[..6]).bold().blue(),
            "...".dimmed(),
            (&id_str[id_str.len() - 6..]).bold().blue()
        )
    }
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let ms = duration.as_millis();

    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{}s", ms / 1000)
    } else if ms < 3_600_000 {
        format!("{}min", ms / 60_000)
    } else if ms < 86_400_000 {
        format!("{}h", ms / 3_600_000)
    } else {
        format!("{}d", ms / 86_400_000)
    }
}
