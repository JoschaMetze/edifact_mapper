/// Type-aware placeholder values for EDIFACT data elements.
///
/// Maps UN/EDIFACT data element IDs to realistic placeholder values
/// that will survive forward/reverse mapping roundtrips.
pub fn placeholder_for_data_element(element_id: &str) -> &'static str {
    match element_id {
        // Party / organization identifiers
        "3039" => "1234567890128", // GLN (13-digit, valid check digit)
        "3055" => "9",             // Responsible agency code (GS1)
        "1131" => "293",           // Code list code (BDEW)

        // Location identifiers
        "3225" => "DE0012345678901234567890123456789012", // MaLo-ID (36-char)

        // Date/time
        "2380" => "20250401", // Date (CCYYMMDD) — will be extended by format code
        "2379" => "303",      // Date/time format qualifier

        // Party name / address
        "3036" => "Mustermann",    // Party name
        "3124" => "c/o Muster",    // Name and address line (C058)
        "3042" => "Musterstrasse", // Street and number
        "3164" => "Musterstadt",   // City name
        "3251" => "12345",         // Postal code
        "3207" => "DE",            // Country code (ISO 3166)
        "3229" => "HE",            // Region code

        // Communication
        "3148" => "test@example.com", // Communication address/number
        "3413" => "0800123456",       // Contact number
        "3412" => "Herr Muster",      // Contact name

        // Document / message references
        "1004" => "GENERATED00001", // Document/message number
        "1154" => "GENERATED00001", // Reference identifier
        "0062" => "GENERATED00001", // Message reference number
        "0020" => "GENERATED00001", // Interchange control reference
        "1050" => "1",              // Sequence number

        // Quantities / measurements
        "6060" => "1",   // Quantity
        "6411" => "KWH", // Measurement unit code
        "6314" => "100", // Measurement value
        "7111" => "1",   // Characteristic value (CAV)

        // Status / process
        "9015" => "Z01", // Status description code
        "4405" => "Z01", // Status code

        // Identification
        "7405" => "Z01",    // Identification code qualifier
        "7402" => "TESTID", // Identification number

        // Item / product
        "7140" => "TESTPRODUCT", // Item identifier
        "5025" => "1",           // Monetary amount

        // Catch-all: return a generic non-empty value
        _ => "X",
    }
}

/// Generate a date/time placeholder appropriate for the DTM format code.
pub fn placeholder_datetime(format_code: Option<&str>) -> &'static str {
    match format_code {
        Some("102") => "20250401",          // CCYYMMDD
        Some("203") => "202504011200",      // CCYYMMDDHHMM
        Some("303") => "20250401120000+00", // CCYYMMDDHHMMSS+offset (with ?+ escaped later)
        Some("602") => "202504",            // CCYYMM
        _ => "20250401",                    // Default to CCYYMMDD
    }
}
