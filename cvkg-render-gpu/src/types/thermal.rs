/// Device thermal state for quality scaling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalState {
    /// Normal operation, no thermal pressure.
    Nominal,
    /// Slight thermal pressure, reduce non-essential effects.
    Fair,
    /// Significant thermal pressure, reduce quality.
    Serious,
    /// Critical thermal pressure, minimal rendering.
    Critical,
}

impl Default for ThermalState {
    fn default() -> Self {
        ThermalState::Nominal
    }
}

impl ThermalState {
    /// Determine thermal state from a normalized temperature reading (0.0-1.0).
    pub fn from_temperature(temp: f32) -> Self {
        if temp < 0.6 {
            ThermalState::Nominal
        } else if temp < 0.75 {
            ThermalState::Fair
        } else if temp < 0.9 {
            ThermalState::Serious
        } else {
            ThermalState::Critical
        }
    }

    /// Returns the quality scale factor for this thermal state.
    pub fn quality_scale(&self) -> f32 {
        match self {
            ThermalState::Nominal => 1.0,
            ThermalState::Fair => 0.75,
            ThermalState::Serious => 0.5,
            ThermalState::Critical => 0.25,
        }
    }

    /// Whether volumetric effects should be enabled at this thermal state.
    pub fn enable_volumetric(&self) -> bool {
        matches!(self, ThermalState::Nominal)
    }

    /// Whether bloom should be enabled at this thermal state.
    pub fn enable_bloom(&self) -> bool {
        matches!(self, ThermalState::Nominal | ThermalState::Fair)
    }

    /// Returns the MSAA sample count for this thermal state.
    pub fn msaa_sample_count(&self) -> u32 {
        match self {
            ThermalState::Nominal => 4,
            ThermalState::Fair => 2,
            ThermalState::Serious | ThermalState::Critical => 1,
        }
    }
}

/// Thermal monitoring configuration.
#[derive(Clone, Copy, Debug)]
pub struct ThermalConfig {
    /// How often to check thermal state (in frames).
    pub check_interval_frames: u32,
    /// Hysteresis: how much the temperature must drop before improving quality.
    pub hysteresis: f32,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            check_interval_frames: 60, // Check once per second at 60fps
            hysteresis: 0.05,
        }
    }
}

#[cfg(test)]
mod p2_27_thermal_tests {
    use super::*;

    #[test]
    fn thermal_state_from_temperature() {
        assert_eq!(ThermalState::from_temperature(0.3), ThermalState::Nominal);
        assert_eq!(ThermalState::from_temperature(0.7), ThermalState::Fair);
        assert_eq!(ThermalState::from_temperature(0.85), ThermalState::Serious);
        assert_eq!(ThermalState::from_temperature(0.95), ThermalState::Critical);
    }

    #[test]
    fn thermal_quality_scale() {
        assert_eq!(ThermalState::Nominal.quality_scale(), 1.0);
        assert_eq!(ThermalState::Fair.quality_scale(), 0.75);
        assert_eq!(ThermalState::Serious.quality_scale(), 0.5);
        assert_eq!(ThermalState::Critical.quality_scale(), 0.25);
    }

    #[test]
    fn thermal_effect_enabling() {
        assert!(ThermalState::Nominal.enable_volumetric());
        assert!(!ThermalState::Fair.enable_volumetric());
        assert!(!ThermalState::Serious.enable_volumetric());

        assert!(ThermalState::Nominal.enable_bloom());
        assert!(ThermalState::Fair.enable_bloom());
        assert!(!ThermalState::Serious.enable_bloom());
    }

    #[test]
    fn thermal_msaa_samples() {
        assert_eq!(ThermalState::Nominal.msaa_sample_count(), 4);
        assert_eq!(ThermalState::Fair.msaa_sample_count(), 2);
        assert_eq!(ThermalState::Serious.msaa_sample_count(), 1);
        assert_eq!(ThermalState::Critical.msaa_sample_count(), 1);
    }

    #[test]
    fn thermal_config_default() {
        let config = ThermalConfig::default();
        assert_eq!(config.check_interval_frames, 60);
        assert_eq!(config.hysteresis, 0.05);
    }
}
