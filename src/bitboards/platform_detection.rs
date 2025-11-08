//! Platform detection and capability detection for bit-scanning optimizations
//!
//! This module provides runtime detection of CPU features and platform capabilities
//! to select the optimal bit-scanning implementation for the current environment.

// Note: Bitboard type will be used in future implementations

/// Supported bit-scanning implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitscanImpl {
    /// Hardware-accelerated bit scanning (native platforms only)
    Hardware,
    /// De Bruijn sequence lookup (fallback)
    DeBruijn,
    /// Generic software implementation (final fallback)
    Software,
}

/// Supported population count implementations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopcountImpl {
    /// Hardware-accelerated population count (native platforms only)
    Hardware,
    /// SWAR (SIMD Within A Register) bit counting fallback
    BitParallel,
    /// Generic software implementation (final fallback)
    Software,
}

/// Supported CPU architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// x86_64 architecture
    X86_64,
    /// ARM architecture
    ARM,
    /// Unknown architecture
    Unknown,
}

/// Platform capabilities and detected features
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    /// x86_64 POPCNT instruction support
    pub has_popcnt: bool,
    /// x86_64 BMI1 instruction support
    pub has_bmi1: bool,
    /// x86_64 BMI2 instruction support
    pub has_bmi2: bool,
    /// Detected architecture
    pub architecture: Architecture,
}

impl Default for PlatformCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

impl PlatformCapabilities {
    /// Detect platform capabilities at runtime
    pub fn detect() -> Self {
        Self::detect_native_capabilities()
    }

    /// Detect capabilities for native platforms
    fn detect_native_capabilities() -> Self {
        let architecture = Self::detect_architecture();

        Self {
            has_popcnt: Self::detect_popcnt_support(),
            has_bmi1: Self::detect_bmi1_support(),
            has_bmi2: Self::detect_bmi2_support(),
            architecture,
        }
    }

    /// Detect CPU architecture
    fn detect_architecture() -> Architecture {
        #[cfg(target_arch = "x86_64")]
        {
            Architecture::X86_64
        }

        #[cfg(target_arch = "aarch64")]
        {
            Architecture::ARM
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Architecture::Unknown
        }
    }

    /// Detect x86_64 POPCNT instruction support
    #[cfg(target_arch = "x86_64")]
    fn detect_popcnt_support() -> bool {
        unsafe {
            use std::arch::x86_64::__cpuid;

            // Check CPUID feature flags for POPCNT support
            let cpuid = __cpuid(1);
            (cpuid.ecx & (1 << 23)) != 0 // POPCNT bit in ECX register
        }
    }

    /// Detect x86_64 BMI1 instruction support
    #[cfg(target_arch = "x86_64")]
    fn detect_bmi1_support() -> bool {
        unsafe {
            use std::arch::x86_64::__cpuid;

            // Check CPUID feature flags for BMI1 support
            let cpuid = __cpuid(7);
            (cpuid.ebx & (1 << 3)) != 0 // BMI1 bit in EBX register
        }
    }

    /// Detect x86_64 BMI2 instruction support
    #[cfg(target_arch = "x86_64")]
    fn detect_bmi2_support() -> bool {
        unsafe {
            use std::arch::x86_64::__cpuid;

            // Check CPUID feature flags for BMI2 support
            let cpuid = __cpuid(7);
            (cpuid.ebx & (1 << 8)) != 0 // BMI2 bit in EBX register
        }
    }

    /// Fallback implementations for non-x86_64 platforms
    #[cfg(not(target_arch = "x86_64"))]
    fn detect_popcnt_support() -> bool {
        false
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn detect_bmi1_support() -> bool {
        false
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn detect_bmi2_support() -> bool {
        false
    }

    /// Get optimal bitscan implementation for this platform
    pub fn get_bitscan_impl(&self) -> BitscanImpl {
        if self.has_bmi1 {
            BitscanImpl::Hardware // Use hardware acceleration when available
        } else {
            BitscanImpl::DeBruijn // Use De Bruijn as fallback
        }
    }

    /// Get optimal popcount implementation for this platform
    pub fn get_popcount_impl(&self) -> PopcountImpl {
        if self.has_popcnt {
            PopcountImpl::Hardware // Use hardware acceleration when available
        } else {
            PopcountImpl::BitParallel // Use SWAR as fallback
        }
    }

    /// Check if platform supports hardware acceleration
    pub fn has_hardware_acceleration(&self) -> bool {
        self.has_popcnt || self.has_bmi1
    }

    /// Get platform summary string
    pub fn get_summary(&self) -> String {
        format!(
            "Architecture: {:?}, POPCNT: {}, BMI1: {}, BMI2: {}",
            self.architecture, self.has_popcnt, self.has_bmi1, self.has_bmi2
        )
    }
}

/// Global platform capabilities instance
static PLATFORM_CAPABILITIES: std::sync::OnceLock<PlatformCapabilities> =
    std::sync::OnceLock::new();

/// Get the global platform capabilities instance
pub fn get_platform_capabilities() -> &'static PlatformCapabilities {
    PLATFORM_CAPABILITIES.get_or_init(PlatformCapabilities::detect)
}

/// Get optimal bitscan implementation for current platform
pub fn get_best_bitscan_impl() -> BitscanImpl {
    get_platform_capabilities().get_bitscan_impl()
}

/// Get optimal popcount implementation for current platform
pub fn get_best_popcount_impl() -> PopcountImpl {
    get_platform_capabilities().get_popcount_impl()
}

/// Check if current platform has hardware acceleration
pub fn has_hardware_support() -> bool {
    get_platform_capabilities().has_hardware_acceleration()
}

/// Get platform summary for debugging
pub fn get_platform_summary() -> String {
    get_platform_capabilities().get_summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_capabilities_detection() {
        let caps = PlatformCapabilities::detect();

        // Basic sanity checks
        assert_ne!(caps.architecture, Architecture::Unknown);
    }

    #[test]
    fn test_implementation_selection() {
        let caps = PlatformCapabilities::detect();

        let bitscan_impl = caps.get_bitscan_impl();
        let popcount_impl = caps.get_popcount_impl();

        // Hardware acceleration should be preferred when available
        if caps.has_bmi1 {
            assert_eq!(bitscan_impl, BitscanImpl::Hardware);
        }

        if caps.has_popcnt {
            assert_eq!(popcount_impl, PopcountImpl::Hardware);
        }
    }

    #[test]
    fn test_global_functions() {
        // Test that global functions work
        let _impl = get_best_bitscan_impl();
        let _impl = get_best_popcount_impl();
        let _has_hw = has_hardware_support();
        let _summary = get_platform_summary();

        // Ensure we get consistent results
        let caps1 = get_platform_capabilities();
        let caps2 = get_platform_capabilities();
        assert_eq!(caps1.architecture, caps2.architecture);
    }

    #[test]
    fn test_platform_summary_format() {
        let summary = get_platform_summary();
        assert!(!summary.is_empty());
        assert!(summary.contains("Architecture:"));
        assert!(summary.contains("POPCNT:"));
        assert!(summary.contains("BMI1:"));
        assert!(summary.contains("BMI2:"));
    }

    #[test]
    fn test_cross_platform_consistency() {
        // Test that detection works on all platforms
        let caps = PlatformCapabilities::detect();

        // Architecture should be detected
        assert_ne!(caps.architecture, Architecture::Unknown);

        // Implementation selection should work
        let _bitscan = caps.get_bitscan_impl();
        let _popcount = caps.get_popcount_impl();

        // Hardware acceleration check should work
        let _has_hw = caps.has_hardware_acceleration();
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_detection_performance() {
        // Test that detection is fast (should be < 1% overhead)
        let iterations = 1000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _caps = PlatformCapabilities::detect();
        }
        let duration = start.elapsed();

        let avg_time_ns = duration.as_nanos() / iterations;
        assert!(
            avg_time_ns < 1_000_000,
            "Detection too slow: {}ns average",
            avg_time_ns
        );
    }

    #[test]
    fn test_global_access_performance() {
        // Test that global access is fast
        let iterations = 10000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _caps = get_platform_capabilities();
            let _impl = get_best_bitscan_impl();
            let _impl = get_best_popcount_impl();
            let _has_hw = has_hardware_support();
        }
        let duration = start.elapsed();

        let avg_time_ns = duration.as_nanos() / iterations;
        assert!(
            avg_time_ns < 1000,
            "Global access too slow: {}ns average",
            avg_time_ns
        );
    }
}
