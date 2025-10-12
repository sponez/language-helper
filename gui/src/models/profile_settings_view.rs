//! Profile settings view model for GUI presentation.

/// View model for displaying profile settings in the GUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSettingsView {
    /// Number of cards shown per training set
    pub cards_per_set: u32,
    /// Method for testing: "manual" or "self_review"
    pub test_answer_method: String,
    /// Number of correct answers in a row to mark card as learned
    pub streak_length: u32,
    /// Selected AI model (if any)
    pub ai_model: Option<String>,
}

impl ProfileSettingsView {
    /// Creates a new ProfileSettingsView with default values.
    pub fn default() -> Self {
        Self {
            cards_per_set: 10,
            test_answer_method: "manual".to_string(),
            streak_length: 5,
            ai_model: None,
        }
    }

    /// Creates a new ProfileSettingsView.
    pub fn new(
        cards_per_set: u32,
        test_answer_method: String,
        streak_length: u32,
        ai_model: Option<String>,
    ) -> Self {
        Self {
            cards_per_set,
            test_answer_method,
            streak_length,
            ai_model,
        }
    }
}
