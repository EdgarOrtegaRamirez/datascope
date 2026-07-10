use crate::profile::DatasetProfile;

/// Render profile as pretty-printed JSON.
pub fn render_json(profile: &DatasetProfile) -> String {
    serde_json::to_string_pretty(profile)
        .unwrap_or_else(|e| format!(r#"{{"error": "failed to serialize profile: {e}"}}"#))
}
