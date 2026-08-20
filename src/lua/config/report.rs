use super::custom::Custom;

#[derive(Debug, Clone, Default)]
pub struct Report {
    entry: Option<Custom>,
    summary: Option<Custom>,
    render: Option<Custom>,
}

impl Report {
    pub fn with_entry(mut self, entry: Option<Custom>) -> Self {
        self.entry = entry;
        self
    }

    pub fn with_summary(mut self, summary: Option<Custom>) -> Self {
        self.summary = summary;
        self
    }

    pub fn with_render(mut self, render: Option<Custom>) -> Self {
        self.render = render;
        self
    }

    pub fn merge(&mut self, other: Report) {
        self.entry = other.entry.or(self.entry.take());
        self.summary = other.summary.or(self.summary.take());
        self.render = other.render.or(self.render.take());
    }

    pub fn entry(&self) -> Option<&Custom> {
        self.entry.as_ref()
    }

    pub fn summary(&self) -> Option<&Custom> {
        self.summary.as_ref()
    }

    pub fn render(&self) -> Option<&Custom> {
        self.render.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merging_only_replaces_the_pieces_the_other_one_carries() {
        let mut report = Report::default()
            .with_entry(Some(Custom::Silent))
            .with_summary(Some(Custom::Text("first".to_string())));

        report.merge(Report::default().with_summary(Some(Custom::Text("second".to_string()))));

        assert!(matches!(report.entry(), Some(Custom::Silent)));
        assert!(matches!(report.summary(), Some(Custom::Text(text)) if text == "second"));
        assert!(report.render().is_none());
    }
}
