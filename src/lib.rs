use serde::{Deserialize, Serialize};

pub mod client;
pub mod collection;
pub mod molecule;
pub mod procedure;

#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    #[serde(rename = "COMPLETE")]
    Complete,
    #[serde(rename = "ERROR")]
    Error,
}

impl Status {
    /// Returns `true` if the status is [`Complete`].
    ///
    /// [`Complete`]: Status::Complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}
