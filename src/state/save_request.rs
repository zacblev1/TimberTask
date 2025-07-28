/// Save request indicating what type of save operation is needed
#[derive(Debug, Clone)]
pub enum SaveRequest {
    /// Save the entire state
    Full,
    /// No save needed
    None,
}

impl SaveRequest {
    /// Check if a save is needed
    pub fn is_needed(&self) -> bool {
        matches!(self, SaveRequest::Full)
    }
}