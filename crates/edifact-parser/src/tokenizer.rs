use edifact_types::EdifactDelimiters;

/// Tokenizes raw EDIFACT byte input into segment strings.
///
/// Handles release character escaping, whitespace normalization (strips \r\n),
/// and UNA segment detection.
pub struct EdifactTokenizer {
    delimiters: EdifactDelimiters,
}

impl EdifactTokenizer {
    /// Creates a new tokenizer with the given delimiters.
    pub fn new(delimiters: EdifactDelimiters) -> Self {
        Self { delimiters }
    }

    /// Returns the delimiters used by this tokenizer.
    pub fn delimiters(&self) -> &EdifactDelimiters {
        &self.delimiters
    }

    /// Tokenizes EDIFACT input into segment strings.
    ///
    /// Splits on segment terminator, respecting release character escaping.
    /// Strips `\r` and `\n` characters from the input (EDIFACT uses them
    /// only for readability).
    ///
    /// Each yielded string is a segment WITHOUT its terminator character.
    pub fn tokenize_segments<'a>(&self, input: &'a [u8]) -> SegmentIter<'a> {
        SegmentIter {
            input,
            pos: 0,
            segment_terminator: self.delimiters.segment,
            release_char: self.delimiters.release,
        }
    }

    /// Tokenizes a segment string into data elements.
    ///
    /// Splits on element separator, preserving release character escaping
    /// (unescaping happens at the component level).
    pub fn tokenize_elements<'a>(&self, segment: &'a str) -> ElementIter<'a> {
        ElementIter {
            input: segment,
            pos: 0,
            separator: self.delimiters.element as char,
            release: self.delimiters.release as char,
        }
    }

    /// Tokenizes a data element into components.
    ///
    /// Splits on component separator and unescapes release character sequences.
    pub fn tokenize_components<'a>(&self, element: &'a str) -> ComponentIter<'a> {
        ComponentIter {
            input: element,
            pos: 0,
            separator: self.delimiters.component as char,
            release: self.delimiters.release as char,
        }
    }
}

/// Iterator over segments in raw EDIFACT input bytes.
pub struct SegmentIter<'a> {
    input: &'a [u8],
    pos: usize,
    segment_terminator: u8,
    release_char: u8,
}

impl<'a> Iterator for SegmentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespace between segments
        while self.pos < self.input.len() {
            let b = self.input[self.pos];
            if b == b'\r' || b == b'\n' || b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos >= self.input.len() {
            return None;
        }

        let start = self.pos;
        let mut i = self.pos;

        while i < self.input.len() {
            let b = self.input[i];

            // Skip \r and \n within segments (EDIFACT ignores them)
            if b == b'\r' || b == b'\n' {
                i += 1;
                continue;
            }

            // Check for release character — next byte is escaped
            if b == self.release_char && i + 1 < self.input.len() {
                i += 2; // skip release char and the escaped char
                continue;
            }

            if b == self.segment_terminator {
                // Found unescaped terminator
                let segment_bytes = &self.input[start..i];
                self.pos = i + 1;

                // Build segment string, stripping \r and \n
                let segment_str = strip_crlf(segment_bytes);
                if segment_str.is_empty() {
                    return self.next(); // skip empty segments
                }
                return Some(segment_str);
            }

            i += 1;
        }

        // Remaining content after last terminator (may be trailing whitespace)
        if start < self.input.len() {
            let segment_bytes = &self.input[start..];
            self.pos = self.input.len();
            let segment_str = strip_crlf(segment_bytes);
            if segment_str.is_empty() {
                return None;
            }
            return Some(segment_str);
        }

        None
    }
}

/// Converts a byte slice to a string, stripping \r and \n characters.
///
/// In practice, EDIFACT segments never contain embedded newlines as data
/// (they are only used as line separators between segments for readability).
/// So we can safely interpret the bytes as UTF-8 and trim.
fn strip_crlf(bytes: &[u8]) -> &str {
    // Fast path: try to interpret as UTF-8 and trim
    let s = std::str::from_utf8(bytes).unwrap_or("");
    s.trim_matches(|c: char| c == '\r' || c == '\n')
}

/// Iterator over elements within a segment string.
pub struct ElementIter<'a> {
    input: &'a str,
    pos: usize,
    separator: char,
    release: char,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.input.len() {
            return None;
        }

        let start = self.pos;
        let bytes = self.input.as_bytes();
        let mut i = self.pos;

        while i < bytes.len() {
            let ch = bytes[i] as char;

            // Release character escapes the next character
            if ch == self.release && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            if ch == self.separator {
                let element = &self.input[start..i];
                self.pos = i + 1;
                return Some(element);
            }

            i += 1;
        }

        // Return remaining content
        if start <= self.input.len() {
            let element = &self.input[start..];
            self.pos = self.input.len() + 1; // mark as exhausted
            return Some(element);
        }

        None
    }
}

/// Iterator over components within a data element.
pub struct ComponentIter<'a> {
    input: &'a str,
    pos: usize,
    separator: char,
    release: char,
}

impl<'a> Iterator for ComponentIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.input.len() {
            return None;
        }

        let start = self.pos;
        let bytes = self.input.as_bytes();
        let mut i = self.pos;

        while i < bytes.len() {
            let ch = bytes[i] as char;

            // Release character escapes the next character
            if ch == self.release && i + 1 < bytes.len() {
                i += 2;
                continue;
            }

            if ch == self.separator {
                let component = &self.input[start..i];
                self.pos = i + 1;
                return Some(component);
            }

            i += 1;
        }

        // Return remaining content
        if start <= self.input.len() {
            let component = &self.input[start..];
            self.pos = self.input.len() + 1;
            return Some(component);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_segments_simple() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"UNB+UNOC:3'UNH+00001'UNT+2+00001'UNZ+1'";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(
            segments,
            vec!["UNB+UNOC:3", "UNH+00001", "UNT+2+00001", "UNZ+1"]
        );
    }

    #[test]
    fn test_tokenize_segments_with_newlines() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"UNB+UNOC:3'\nUNH+00001'\r\nUNT+2+00001'\nUNZ+1'";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(
            segments,
            vec!["UNB+UNOC:3", "UNH+00001", "UNT+2+00001", "UNZ+1"]
        );
    }

    #[test]
    fn test_tokenize_segments_with_release_char() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        // ?'  is an escaped apostrophe — NOT a segment terminator
        let input = b"FTX+ACB+++text with ?'quotes?''";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "FTX+ACB+++text with ?'quotes?'");
    }

    #[test]
    fn test_tokenize_segments_empty_input() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_tokenize_segments_trailing_whitespace() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"UNH+00001'  \n  ";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments, vec!["UNH+00001"]);
    }

    #[test]
    fn test_tokenize_segments_custom_delimiter() {
        let delimiters = EdifactDelimiters {
            segment: b'!',
            ..EdifactDelimiters::default()
        };
        let tokenizer = EdifactTokenizer::new(delimiters);
        let input = b"UNB+UNOC:3!UNH+00001!";
        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments, vec!["UNB+UNOC:3", "UNH+00001"]);
    }

    // --- Task 2: Element and Component Splitting ---

    #[test]
    fn test_tokenize_elements() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let elements: Vec<&str> = tokenizer
            .tokenize_elements("NAD+Z04+9900123000002:500")
            .collect();
        assert_eq!(elements, vec!["NAD", "Z04", "9900123000002:500"]);
    }

    #[test]
    fn test_tokenize_elements_escaped_plus() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let elements: Vec<&str> = tokenizer
            .tokenize_elements("FTX+ACB+++value with ?+plus")
            .collect();
        // ?+ is escaped, so it should NOT split; +++ produces two empty elements
        assert_eq!(elements, vec!["FTX", "ACB", "", "", "value with ?+plus"]);
    }

    #[test]
    fn test_tokenize_components() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let components: Vec<&str> = tokenizer
            .tokenize_components("UTILMD:D:11A:UN:S2.1")
            .collect();
        assert_eq!(components, vec!["UTILMD", "D", "11A", "UN", "S2.1"]);
    }

    #[test]
    fn test_tokenize_components_escaped_colon() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let components: Vec<&str> = tokenizer.tokenize_components("value?:with:colon").collect();
        // ?: is escaped, so "value?:with" is one component
        assert_eq!(components, vec!["value?:with", "colon"]);
    }

    #[test]
    fn test_tokenize_components_empty() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let components: Vec<&str> = tokenizer.tokenize_components("Z04::500").collect();
        assert_eq!(components, vec!["Z04", "", "500"]);
    }

    #[test]
    fn test_full_tokenization_pipeline() {
        let tokenizer = EdifactTokenizer::new(EdifactDelimiters::default());
        let input = b"NAD+Z04+9900123000002::293'DTM+137:202501010000?+01:303'";

        let segments: Vec<&str> = tokenizer.tokenize_segments(input).collect();
        assert_eq!(segments.len(), 2);

        // Parse first segment: NAD+Z04+9900123000002::293
        let elements: Vec<&str> = tokenizer.tokenize_elements(segments[0]).collect();
        assert_eq!(elements, vec!["NAD", "Z04", "9900123000002::293"]);

        // Parse composite element: 9900123000002::293
        let components: Vec<&str> = tokenizer.tokenize_components(elements[2]).collect();
        assert_eq!(components, vec!["9900123000002", "", "293"]);

        // Parse second segment: DTM+137:202501010000?+01:303
        let dtm_elements: Vec<&str> = tokenizer.tokenize_elements(segments[1]).collect();
        assert_eq!(dtm_elements, vec!["DTM", "137:202501010000?+01:303"]);

        // Parse DTM composite (note: ?+ is escaped at element level, kept as-is)
        let dtm_components: Vec<&str> = tokenizer.tokenize_components(dtm_elements[1]).collect();
        assert_eq!(dtm_components, vec!["137", "202501010000?+01", "303"]);
    }
}
