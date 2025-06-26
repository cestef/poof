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
