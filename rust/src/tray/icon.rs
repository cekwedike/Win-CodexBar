//! Tray icon types
//!
//! Provides usage level and badge types for tray icon rendering

/// Usage status level for icon color
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsageLevel {
    /// 0-50% used - green
    Low,
    /// 50-80% used - yellow
    Medium,
    /// 80-95% used - orange
    High,
    /// 95-100% used - red
    Critical,
    /// Unknown/error state - gray
    Unknown,
}

impl UsageLevel {
    pub fn from_percent(percent: f64) -> Self {
        match percent {
            p if p < 50.0 => UsageLevel::Low,
            p if p < 80.0 => UsageLevel::Medium,
            p if p < 95.0 => UsageLevel::High,
            _ => UsageLevel::Critical,
        }
    }

    /// Get RGB color for this usage level
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            UsageLevel::Low => (76, 175, 80),       // Green
            UsageLevel::Medium => (255, 193, 7),    // Yellow/Amber
            UsageLevel::High => (255, 152, 0),      // Orange
            UsageLevel::Critical => (244, 67, 54),  // Red
            UsageLevel::Unknown => (158, 158, 158), // Gray
        }
    }
}

/// Badge type for status indicators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BadgeType {
    /// Warning indicator (yellow)
    Warning,
    /// Error/incident indicator (red)
    Incident,
    /// No badge
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_level_from_percent() {
        assert_eq!(UsageLevel::from_percent(0.0), UsageLevel::Low);
        assert_eq!(UsageLevel::from_percent(25.0), UsageLevel::Low);
        assert_eq!(UsageLevel::from_percent(49.9), UsageLevel::Low);
        assert_eq!(UsageLevel::from_percent(50.0), UsageLevel::Medium);
        assert_eq!(UsageLevel::from_percent(79.9), UsageLevel::Medium);
        assert_eq!(UsageLevel::from_percent(80.0), UsageLevel::High);
        assert_eq!(UsageLevel::from_percent(94.9), UsageLevel::High);
        assert_eq!(UsageLevel::from_percent(95.0), UsageLevel::Critical);
        assert_eq!(UsageLevel::from_percent(100.0), UsageLevel::Critical);
    }

    #[test]
    fn test_usage_level_color() {
        // Just verify colors are returned as RGB tuples
        let (r, g, _b) = UsageLevel::Low.color();
        assert!(r < g); // Green should be dominant for low usage

        let (r, g, b) = UsageLevel::Critical.color();
        assert!(r > g && r > b); // Red should be dominant for critical
    }

    #[test]
    fn test_badge_type_equality() {
        assert_eq!(BadgeType::None, BadgeType::None);
        assert_eq!(BadgeType::Warning, BadgeType::Warning);
        assert_eq!(BadgeType::Incident, BadgeType::Incident);
        assert_ne!(BadgeType::None, BadgeType::Warning);
    }
}
