use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DisplayMode { #[default] Inherit, Windowed, Fullscreen }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaunchSettings {
    pub max_memory_mib: Option<u32>,
    pub display_mode: DisplayMode,
    pub width: u32,
    pub height: u32,
    pub remember_maximized: bool,
}
impl Default for LaunchSettings {
    fn default() -> Self {
        Self { max_memory_mib: None, display_mode: DisplayMode::Inherit, width: 1280, height: 720, remember_maximized: false }
    }
}
impl LaunchSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_memory_mib.is_some_and(|v| v < 103) { return Err("Maximum memory must be at least 0.1 GB".into()); }
        if !(320..=3840).contains(&self.width) || !(200..=2400).contains(&self.height) { return Err("Invalid window dimensions".into()); }
        Ok(())
    }
    pub fn validate_available_memory(&self, available: u64) -> Result<(), String> {
        self.validate()?;
        if self.max_memory_mib.is_some_and(|m| u64::from(m) * 1024 * 1024 > available) {
            return Err("Maximum memory exceeds currently available memory".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_bounds_and_current_available_memory() {
        let mut settings = LaunchSettings::default();
        assert!(settings.validate_available_memory(0).is_ok());
        settings.max_memory_mib = Some(1024);
        assert!(settings.validate_available_memory(1024 * 1024 * 1024).is_ok());
        assert!(settings.validate_available_memory(1024 * 1024 * 1024 - 1).is_err());
        settings.max_memory_mib = Some(0);
        assert!(settings.validate().is_err());
        settings.max_memory_mib = None;
        settings.width = 0;
        assert!(settings.validate().is_err());
    }
}
