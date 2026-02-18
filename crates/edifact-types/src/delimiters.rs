/// Error when parsing a UNA service string advice segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaParseError {
    /// UNA segment must be exactly 9 bytes.
    InvalidLength { expected: usize, actual: usize },
    /// UNA segment must start with "UNA".
    InvalidPrefix,
}

impl std::fmt::Display for UnaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "UNA segment must be exactly {expected} bytes, got {actual}"
                )
            }
            Self::InvalidPrefix => write!(f, "UNA segment must start with 'UNA'"),
        }
    }
}

impl std::error::Error for UnaParseError {}

/// EDIFACT delimiter characters.
///
/// The six characters that control EDIFACT message structure. When no UNA
/// service string advice is present, the standard defaults apply:
/// - Component separator: `:` (colon)
/// - Element separator: `+` (plus)
/// - Decimal mark: `.` (period)
/// - Release character: `?` (question mark)
/// - Segment terminator: `'` (apostrophe)
/// - Reserved: ` ` (space)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdifactDelimiters {
    /// Component data element separator (default: `:`).
    pub component: u8,
    /// Data element separator (default: `+`).
    pub element: u8,
    /// Decimal mark (default: `.`).
    pub decimal: u8,
    /// Release character / escape (default: `?`).
    pub release: u8,
    /// Segment terminator (default: `'`).
    pub segment: u8,
    /// Reserved for future use (default: ` `).
    pub reserved: u8,
}

impl Default for EdifactDelimiters {
    fn default() -> Self {
        Self {
            component: b':',
            element: b'+',
            decimal: b'.',
            release: b'?',
            segment: b'\'',
            reserved: b' ',
        }
    }
}

impl EdifactDelimiters {
    /// Standard EDIFACT delimiters (when no UNA segment is present).
    pub const STANDARD: Self = Self {
        component: b':',
        element: b'+',
        decimal: b'.',
        release: b'?',
        segment: b'\'',
        reserved: b' ',
    };

    /// Parse delimiters from a UNA service string advice segment.
    ///
    /// The UNA segment is exactly 9 bytes: `UNA` followed by 6 delimiter characters.
    /// Format: `UNA<component><element><decimal><release><reserved><terminator>`
    ///
    /// # Errors
    ///
    /// Returns an error if the input is not exactly 9 bytes or does not start with `UNA`.
    pub fn from_una(una: &[u8]) -> Result<Self, UnaParseError> {
        if una.len() != 9 {
            return Err(UnaParseError::InvalidLength {
                expected: 9,
                actual: una.len(),
            });
        }

        if &una[0..3] != b"UNA" {
            return Err(UnaParseError::InvalidPrefix);
        }

        // UNA format positions:
        // 0-2: "UNA"
        // 3: component separator
        // 4: element separator
        // 5: decimal mark
        // 6: release character
        // 7: reserved
        // 8: segment terminator
        Ok(Self {
            component: una[3],
            element: una[4],
            decimal: una[5],
            release: una[6],
            reserved: una[7],
            segment: una[8],
        })
    }

    /// Detect delimiters from an EDIFACT message.
    ///
    /// If the message starts with a UNA segment, parses delimiters from it.
    /// Otherwise, returns the standard defaults.
    ///
    /// Returns `(has_una, delimiters)`.
    pub fn detect(input: &[u8]) -> (bool, Self) {
        if input.len() >= 9 && &input[0..3] == b"UNA" {
            match Self::from_una(&input[0..9]) {
                Ok(d) => (true, d),
                Err(_) => (false, Self::default()),
            }
        } else {
            (false, Self::default())
        }
    }

    /// Formats the delimiters as a UNA service string advice segment.
    ///
    /// Returns the 9-byte UNA string: `UNA:+.? '`
    pub fn to_una_string(&self) -> String {
        format!(
            "UNA{}{}{}{}{}{}",
            self.component as char,
            self.element as char,
            self.decimal as char,
            self.release as char,
            self.reserved as char,
            self.segment as char,
        )
    }
}

impl std::fmt::Display for EdifactDelimiters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UNA{}{}{}{}{}{}",
            self.component as char,
            self.element as char,
            self.decimal as char,
            self.release as char,
            self.reserved as char,
            self.segment as char,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_delimiters() {
        let d = EdifactDelimiters::default();
        assert_eq!(d.component, b':');
        assert_eq!(d.element, b'+');
        assert_eq!(d.decimal, b'.');
        assert_eq!(d.release, b'?');
        assert_eq!(d.segment, b'\'');
        assert_eq!(d.reserved, b' ');
    }

    #[test]
    fn test_delimiters_equality() {
        let a = EdifactDelimiters::default();
        let b = EdifactDelimiters::default();
        assert_eq!(a, b);
    }

    #[test]
    fn test_delimiters_debug() {
        let d = EdifactDelimiters::default();
        let debug = format!("{:?}", d);
        assert!(debug.contains("EdifactDelimiters"));
    }

    #[test]
    fn test_from_una_standard() {
        let una = b"UNA:+.? '";
        let d = EdifactDelimiters::from_una(una).unwrap();
        assert_eq!(d, EdifactDelimiters::default());
    }

    #[test]
    fn test_from_una_custom_delimiters() {
        let una = b"UNA;*.# |";
        let d = EdifactDelimiters::from_una(una).unwrap();
        assert_eq!(d.component, b';');
        assert_eq!(d.element, b'*');
        assert_eq!(d.decimal, b'.');
        assert_eq!(d.release, b'#');
        assert_eq!(d.reserved, b' ');
        assert_eq!(d.segment, b'|');
    }

    #[test]
    fn test_from_una_too_short() {
        let una = b"UNA:+.";
        assert!(EdifactDelimiters::from_una(una).is_err());
    }

    #[test]
    fn test_from_una_wrong_prefix() {
        let una = b"XXX:+.? '";
        assert!(EdifactDelimiters::from_una(una).is_err());
    }

    #[test]
    fn test_detect_with_una() {
        let input = b"UNA:+.? 'UNB+UNOC:3+sender+recipient'";
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        assert!(has_una);
        assert_eq!(delimiters, EdifactDelimiters::default());
    }

    #[test]
    fn test_detect_without_una() {
        let input = b"UNB+UNOC:3+sender+recipient'";
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        assert!(!has_una);
        assert_eq!(delimiters, EdifactDelimiters::default());
    }

    #[test]
    fn test_detect_empty_input() {
        let input = b"";
        let (has_una, delimiters) = EdifactDelimiters::detect(input);
        assert!(!has_una);
        assert_eq!(delimiters, EdifactDelimiters::default());
    }

    #[test]
    fn test_una_roundtrip() {
        let original = EdifactDelimiters {
            component: b';',
            element: b'*',
            decimal: b',',
            release: b'#',
            segment: b'!',
            reserved: b' ',
        };
        let una_string = original.to_una_string();
        let parsed = EdifactDelimiters::from_una(una_string.as_bytes()).unwrap();
        assert_eq!(original, parsed);
    }
}
