use crate::Level;
use rosrust_msg::diagnostic_msgs::{DiagnosticStatus, KeyValue};

/// Higher level description of an individual diagnostic status.
#[derive(Clone)]
pub struct Status {
    /// Level of the operation.
    pub level: Level,
    /// A description of the test/component that is being reporting.
    pub name: String,
    /// A description of the status.
    pub message: String,
    /// A hardware unique string.
    pub hardware_id: String,
    /// An array of values associated with the status.
    pub values: Vec<KeyValue>,
}

impl Default for Status {
    /// Creates a status with an OK level and empty message.
    #[inline]
    fn default() -> Self {
        Self::new(Level::Ok, "")
    }
}

impl Status {
    /// Creates a status with the given level and message.
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

    /// Fills out the level and message fields of the status.
    pub fn set_summary(&mut self, level: Level, message: impl std::string::ToString) {
        self.level = level;
        self.message = message.to_string();
    }

    /// Copies the level and message fields from another status.
    #[inline]
    pub fn copy_summary(&mut self, other: &Status) {
        self.set_summary(other.level, &other.message)
    }

    /// Clears the summary, setting the level to OK and making the message empty.
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

    /// Merges a level and message with the existing ones.
    ///
    /// It is sometimes useful to merge two status messages. In that case,
    /// the key value pairs can be unioned, but the level and summary message
    /// have to be merged more intelligently. This function does the merge in
    /// an intelligent manner, combining the summary in this instance with the
    /// passed in level and message.
    ///
    /// The combined level is the greater of the two levels to be merged.
    /// If only one level is OK, and the other is not OK,
    /// the message for the OK level is discarded.
    ///
    /// Otherwise, the messages are combined with a semicolon separator.
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

    /// Merges the passed in status with this status by merging the level and message.
    ///
    /// Look at `merge_summary` for more information.
    #[inline]
    pub fn merge_summary_with(&mut self, other: &Status) {
        self.merge_summary(other.level, &other.message)
    }

    /// Adds a keyu-value pair.
    ///
    /// Any value that implements `ToString` can be added easily this way.
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
