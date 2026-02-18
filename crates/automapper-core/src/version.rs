//! Format version dispatch via marker types and the VersionConfig trait.
//!
//! The hybrid approach: traits for zero-cost compile-time dispatch on the
//! hot path, enum boundary at the runtime entry point.
//!
//! See design doc section 5 (Format Version Dispatch).

use std::marker::PhantomData;

use crate::traits::FormatVersion;

/// Marker type for format version April 2025.
pub struct FV2504;

/// Marker type for format version October 2025.
pub struct FV2510;

/// Associates mapper types with a format version at compile time.
///
/// Each format version implements this trait with its own set of mapper types.
/// This provides zero-cost dispatch for the hot path (no dynamic dispatch).
/// The `create_coordinator()` function uses the `FormatVersion` enum to select
/// the right `VersionConfig` at runtime.
///
/// # Example
///
/// ```ignore
/// impl VersionConfig for FV2504 {
///     const VERSION: FormatVersion = FormatVersion::FV2504;
///     type MarktlokationMapper = marktlokation::MapperV2504;
///     // ... one associated type per entity mapper
/// }
/// ```
pub trait VersionConfig: Send + 'static {
    /// The format version this config represents.
    const VERSION: FormatVersion;

    // Associated mapper types will be added as mappers are implemented.
    // For now we define the trait structure without requiring concrete types.
}

impl VersionConfig for FV2504 {
    const VERSION: FormatVersion = FormatVersion::FV2504;
}

impl VersionConfig for FV2510 {
    const VERSION: FormatVersion = FormatVersion::FV2510;
}

/// Helper struct that carries a version config type parameter.
///
/// Used by `UtilmdCoordinator<V>` to associate with a specific version
/// without storing version-specific data.
#[derive(Debug)]
pub struct VersionPhantom<V: VersionConfig> {
    _marker: PhantomData<V>,
}

impl<V: VersionConfig> VersionPhantom<V> {
    /// Creates a new VersionPhantom.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns the format version.
    pub fn version(&self) -> FormatVersion {
        V::VERSION
    }
}

impl<V: VersionConfig> Default for VersionPhantom<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fv2504_version_config() {
        assert_eq!(FV2504::VERSION, FormatVersion::FV2504);
    }

    #[test]
    fn test_fv2510_version_config() {
        assert_eq!(FV2510::VERSION, FormatVersion::FV2510);
    }

    #[test]
    fn test_version_phantom_fv2504() {
        let phantom = VersionPhantom::<FV2504>::new();
        assert_eq!(phantom.version(), FormatVersion::FV2504);
    }

    #[test]
    fn test_version_phantom_fv2510() {
        let phantom = VersionPhantom::<FV2510>::default();
        assert_eq!(phantom.version(), FormatVersion::FV2510);
    }

    #[test]
    fn test_version_config_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<FV2504>();
        assert_send::<FV2510>();
    }

    #[test]
    fn test_version_config_static_lifetime() {
        fn assert_static<T: 'static>() {}
        assert_static::<FV2504>();
        assert_static::<FV2510>();
    }

    /// Verify that generics parameterized by VersionConfig work correctly.
    #[test]
    fn test_generic_over_version_config() {
        fn get_version_str<V: VersionConfig>() -> &'static str {
            V::VERSION.as_str()
        }

        assert_eq!(get_version_str::<FV2504>(), "FV2504");
        assert_eq!(get_version_str::<FV2510>(), "FV2510");
    }
}
