use crate::msg::diagnostic_msgs::{DiagnosticStatus, KeyValue};
use crate::Level;

#[derive(Clone)]
pub struct Status {
    pub level: Level,
    pub name: String,
    pub message: String,
    pub hardware_id: String,
    pub values: Vec<KeyValue>,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            level: Level::Error,
            name: "".to_string(),
            message: "".to_string(),
            hardware_id: "".to_string(),
            values: vec![],
        }
    }
}

impl Status {
    pub fn set_summary(&mut self, level: Level, message: &str) {
        self.level = level;
        self.message = message.into();
    }

    #[inline]
    pub fn copy_summary(&mut self, other: &Status) {
        self.set_summary(other.level, &other.message)
    }

    #[inline]
    pub fn clear_summary(&mut self) {
        self.set_summary(Level::Ok, "")
    }

    fn merge_messages(&mut self, message: &str) {
        if message.is_empty() {
            return;
        }

        self.message = if self.message.is_empty() {
            message.into()
        } else {
            format!("{}; {}", self.message, message)
        };
    }

    pub fn merge_summary(&mut self, level: Level, message: &str) {
        if level as i8 > self.level as i8 {
            self.level = level;
        }
        match (self.level, level) {
            (Level::Ok, Level::Ok) => self.merge_messages(message),
            (Level::Ok, _) => self.message = message.into(),
            (_, Level::Ok) => {}
            _ => self.merge_messages(message),
        }
    }

    #[inline]
    pub fn merge_summary_with(&mut self, other: &Status) {
        self.merge_summary(other.level, &other.message)
    }

    pub fn add(&mut self, key: String, value: impl std::string::ToString) {
        let value = value.to_string();
        self.values.push(KeyValue { key, value });
    }
}

impl Into<DiagnosticStatus> for Status {
    fn into(self) -> DiagnosticStatus {
        DiagnosticStatus {
            level: self.level as i8,
            name: self.name,
            message: self.message,
            hardware_id: self.hardware_id,
            values: self.values,
        }
    }
}
