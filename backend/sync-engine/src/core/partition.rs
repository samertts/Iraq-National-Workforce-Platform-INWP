use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionKey {
    pub ministry_id: uuid::Uuid,
    pub entity_type: String,
    pub time_bucket: String,
}

impl PartitionKey {
    pub fn new(ministry_id: uuid::Uuid, entity_type: impl Into<String>, time_bucket: impl Into<String>) -> Self {
        Self {
            ministry_id,
            entity_type: entity_type.into(),
            time_bucket: time_bucket.into(),
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(3, '/').collect();
        if parts.len() != 3 {
            return None;
        }
        let ministry_id = uuid::Uuid::parse_str(parts[0]).ok()?;
        Some(Self {
            ministry_id,
            entity_type: parts[1].to_string(),
            time_bucket: parts[2].to_string(),
        })
    }

    pub fn entity_prefix(ministry_id: uuid::Uuid, entity_type: &str) -> String {
        format!("{}/{}", ministry_id, entity_type)
    }

    pub fn ministry_prefix(ministry_id: uuid::Uuid) -> String {
        ministry_id.to_string()
    }

    pub fn is_child_of(&self, prefix: &str) -> bool {
        self.to_string().starts_with(prefix)
    }
}

impl fmt::Display for PartitionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.ministry_id, self.entity_type, self.time_bucket)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeBucket {
    Yearly(u32),
    Monthly(u32, u32),
    Daily(u32, u32, u32),
}

impl TimeBucket {
    pub fn current_monthly() -> Self {
        let now = chrono::Utc::now();
        Self::Monthly(now.year() as u32, now.month())
    }

    pub fn current_daily() -> Self {
        let now = chrono::Utc::now();
        Self::Daily(now.year() as u32, now.month(), now.day())
    }
}

impl fmt::Display for TimeBucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yearly(y) => write!(f, "Y{:04}", y),
            Self::Monthly(y, m) => write!(f, "{:04}-{:02}", y, m),
            Self::Daily(y, m, d) => write!(f, "{:04}-{:02}-{:02}", y, m, d),
        }
    }
}
