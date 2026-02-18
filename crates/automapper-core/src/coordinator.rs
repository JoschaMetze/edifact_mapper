//! Coordinator trait and runtime entry point.
//!
//! The coordinator orchestrates mappers during EDIFACT processing. It implements
//! `EdifactHandler` from the parser crate and routes segments to registered
//! mappers. The `create_coordinator()` function is the runtime entry point
//! that selects the correct version-parameterized coordinator.
//!
//! See design doc section 5 (Coordinator).

use bo4e_extensions::UtilmdTransaktion;
use edifact_parser::EdifactHandler;

use crate::error::AutomapperError;
use crate::traits::FormatVersion;

/// Orchestrates mappers during EDIFACT processing.
///
/// A coordinator implements `EdifactHandler` (from the parser crate) and
/// exposes `parse()` and `generate()` for bidirectional conversion.
///
/// Mirrors C# `CoordinatorBase`.
pub trait Coordinator: EdifactHandler + Send {
    /// Parses an EDIFACT interchange and returns the extracted transactions.
    ///
    /// This is the main forward-mapping entry point. It feeds the input through
    /// the streaming parser with `self` as the handler, then collects all
    /// completed transactions.
    fn parse(&mut self, input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError>;

    /// Generates EDIFACT bytes from a transaction.
    ///
    /// This is the main reverse-mapping entry point. It takes a transaction
    /// and serializes it back to EDIFACT format.
    fn generate(&self, transaktion: &UtilmdTransaktion) -> Result<Vec<u8>, AutomapperError>;

    /// Returns the format version this coordinator handles.
    fn format_version(&self) -> FormatVersion;
}

/// Creates a coordinator for the specified format version.
///
/// This is the **runtime entry point** -- the enum boundary where dynamic
/// dispatch begins. Internally, the returned `Box<dyn Coordinator>` contains
/// a `UtilmdCoordinator<FV2504>` or `UtilmdCoordinator<FV2510>` with
/// compile-time dispatched mappers.
///
/// # Example
///
/// ```ignore
/// let fv = FormatVersion::FV2504;
/// let mut coord = create_coordinator(fv);
/// let transactions = coord.parse(edifact_bytes)?;
/// ```
pub fn create_coordinator(fv: FormatVersion) -> Result<Box<dyn Coordinator>, AutomapperError> {
    match fv {
        FormatVersion::FV2504 => Ok(Box::new(StubCoordinator::new(FormatVersion::FV2504))),
        FormatVersion::FV2510 => Ok(Box::new(StubCoordinator::new(FormatVersion::FV2510))),
    }
}

/// Detects the format version from EDIFACT input.
///
/// Scans for UNH segment and extracts the message version from the
/// message identifier composite (element 1, components 0-4).
/// Returns `None` if the format version cannot be determined.
pub fn detect_format_version(input: &[u8]) -> Option<FormatVersion> {
    let input_str = std::str::from_utf8(input).ok()?;

    // Look for UNH segment to find message type identifier
    // UNH+ref+UTILMD:D:11A:UN:S2.1' -- the S2.1 suffix indicates version
    // For now, we use a simple heuristic based on the message version
    if input_str.contains("S2.1") || input_str.contains("FV2504") {
        Some(FormatVersion::FV2504)
    } else if input_str.contains("S2.2") || input_str.contains("FV2510") {
        Some(FormatVersion::FV2510)
    } else {
        // Default to FV2504 if we can detect it's a UTILMD message
        if input_str.contains("UTILMD") {
            Some(FormatVersion::FV2504)
        } else {
            None
        }
    }
}

/// Stub coordinator used until real UtilmdCoordinator is implemented in Epic 7.
///
/// This allows the trait, create_coordinator(), and tests to work while
/// the full mapper infrastructure is being built.
struct StubCoordinator {
    fv: FormatVersion,
}

impl StubCoordinator {
    fn new(fv: FormatVersion) -> Self {
        Self { fv }
    }
}

impl EdifactHandler for StubCoordinator {}

impl Coordinator for StubCoordinator {
    fn parse(&mut self, _input: &[u8]) -> Result<Vec<UtilmdTransaktion>, AutomapperError> {
        Ok(Vec::new())
    }

    fn generate(&self, _transaktion: &UtilmdTransaktion) -> Result<Vec<u8>, AutomapperError> {
        Ok(Vec::new())
    }

    fn format_version(&self) -> FormatVersion {
        self.fv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_coordinator_fv2504() {
        let coord = create_coordinator(FormatVersion::FV2504).unwrap();
        assert_eq!(coord.format_version(), FormatVersion::FV2504);
    }

    #[test]
    fn test_create_coordinator_fv2510() {
        let coord = create_coordinator(FormatVersion::FV2510).unwrap();
        assert_eq!(coord.format_version(), FormatVersion::FV2510);
    }

    #[test]
    fn test_stub_coordinator_parse_returns_empty() {
        let mut coord = create_coordinator(FormatVersion::FV2504).unwrap();
        let result = coord.parse(b"").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_stub_coordinator_generate_returns_empty() {
        let coord = create_coordinator(FormatVersion::FV2504).unwrap();
        let tx = UtilmdTransaktion::default();
        let result = coord.generate(&tx).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_detect_format_version_fv2504() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN:S2.1'";
        assert_eq!(detect_format_version(input), Some(FormatVersion::FV2504));
    }

    #[test]
    fn test_detect_format_version_fv2510() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN:S2.2'";
        assert_eq!(detect_format_version(input), Some(FormatVersion::FV2510));
    }

    #[test]
    fn test_detect_format_version_utilmd_default() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+UTILMD:D:11A:UN'";
        assert_eq!(detect_format_version(input), Some(FormatVersion::FV2504));
    }

    #[test]
    fn test_detect_format_version_unknown() {
        let input = b"UNA:+.? 'UNB+UNOC:3+S+R'UNH+001+APERAK:D:11A:UN'";
        assert_eq!(detect_format_version(input), None);
    }

    #[test]
    fn test_coordinator_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Box<dyn Coordinator>>();
    }
}
