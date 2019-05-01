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
    #[inline]
    fn default() -> Self {
        Self::new(Level::Ok, "")
    }
}

impl Status {
    #[inline]
    pub fn new(level: Level, message: &str) -> Self {
        let message = message.to_string();
        Self {
            level,
            name: "".to_string(),
            message,
            hardware_id: "".to_string(),
            values: vec![],
        }
    }

    pub fn set_summary(&mut self, level: Level, message: impl std::string::ToString) {
        self.level = level;
        self.message = message.to_string();
    }

    #[inline]
    pub fn copy_summary(&mut self, other: &Status) {
        self.set_summary(other.level, other.message.clone())
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
        match (self.level, level) {
            (Level::Ok, Level::Ok) => self.merge_messages(message),
            (Level::Ok, _) => self.message = message.into(),
            (_, Level::Ok) => {}
            _ => self.merge_messages(message),
        }
        if level as i8 > self.level as i8 {
            self.level = level;
        }
    }

    #[inline]
    pub fn merge_summary_with(&mut self, other: &Status) {
        self.merge_summary(other.level, &other.message)
    }

    pub fn clear_values(&mut self) {
        self.values.clear();
    }

    pub fn add(&mut self, key: impl std::string::ToString, value: impl std::string::ToString) {
        let key = key.to_string();
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
