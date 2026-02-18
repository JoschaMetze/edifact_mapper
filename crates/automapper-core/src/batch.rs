//! Batch processing with rayon parallelism.
//!
//! Each message gets its own coordinator instance -- no shared mutable state,
//! perfect for data-parallel processing with rayon.
//!
//! See design doc section 5 (Batch Processing).

use rayon::prelude::*;

use bo4e_extensions::UtilmdTransaktion;

use crate::coordinator::create_coordinator;
use crate::error::AutomapperError;
use crate::traits::FormatVersion;

/// Converts multiple EDIFACT interchanges in parallel using rayon.
///
/// Each input gets its own coordinator instance for full isolation.
/// Results are returned in the same order as inputs.
///
/// # Example
///
/// ```ignore
/// let inputs: Vec<&[u8]> = load_edifact_files();
/// let results = convert_batch(&inputs, FormatVersion::FV2504);
/// for result in results {
///     match result {
///         Ok(transactions) => process(transactions),
///         Err(e) => log_error(e),
///     }
/// }
/// ```
pub fn convert_batch(
    inputs: &[&[u8]],
    fv: FormatVersion,
) -> Vec<Result<Vec<UtilmdTransaktion>, AutomapperError>> {
    inputs
        .par_iter()
        .map(|input| {
            let mut coord = create_coordinator(fv)?;
            coord.parse(input)
        })
        .collect()
}

/// Converts multiple EDIFACT interchanges sequentially (for comparison/testing).
pub fn convert_sequential(
    inputs: &[&[u8]],
    fv: FormatVersion,
) -> Vec<Result<Vec<UtilmdTransaktion>, AutomapperError>> {
    inputs
        .iter()
        .map(|input| {
            let mut coord = create_coordinator(fv)?;
            coord.parse(input)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MSG1: &[u8] = b"UNA:+.? 'UNB+UNOC:3+S+R+D:T+R1'UNH+M1+UTILMD:D:11A:UN:S2.1'BGM+E03+D1'IDE+24+TX1'LOC+Z16+MALO1'UNT+4+M1'UNZ+1+R1'";
    const MSG2: &[u8] = b"UNA:+.? 'UNB+UNOC:3+S+R+D:T+R2'UNH+M2+UTILMD:D:11A:UN:S2.1'BGM+E03+D2'IDE+24+TX2'LOC+Z16+MALO2'UNT+4+M2'UNZ+1+R2'";
    const MSG3: &[u8] = b"UNA:+.? 'UNB+UNOC:3+S+R+D:T+R3'UNH+M3+UTILMD:D:11A:UN:S2.1'BGM+E03+D3'IDE+24+TX3'LOC+Z17+MELO1'UNT+4+M3'UNZ+1+R3'";

    #[test]
    fn test_convert_batch_multiple() {
        let inputs: Vec<&[u8]> = vec![MSG1, MSG2, MSG3];
        let results = convert_batch(&inputs, FormatVersion::FV2504);

        assert_eq!(results.len(), 3);

        let tx1 = results[0].as_ref().unwrap();
        assert_eq!(tx1.len(), 1);
        assert_eq!(tx1[0].transaktions_id, "TX1");

        let tx2 = results[1].as_ref().unwrap();
        assert_eq!(tx2[0].transaktions_id, "TX2");

        let tx3 = results[2].as_ref().unwrap();
        assert_eq!(tx3[0].transaktions_id, "TX3");
    }

    #[test]
    fn test_convert_batch_empty() {
        let inputs: Vec<&[u8]> = vec![];
        let results = convert_batch(&inputs, FormatVersion::FV2504);
        assert!(results.is_empty());
    }

    #[test]
    fn test_convert_batch_matches_sequential() {
        let inputs: Vec<&[u8]> = vec![MSG1, MSG2, MSG3];

        let parallel = convert_batch(&inputs, FormatVersion::FV2504);
        let sequential = convert_sequential(&inputs, FormatVersion::FV2504);

        assert_eq!(parallel.len(), sequential.len());
        for (p, s) in parallel.iter().zip(sequential.iter()) {
            let p_tx = p.as_ref().unwrap();
            let s_tx = s.as_ref().unwrap();
            assert_eq!(p_tx.len(), s_tx.len());
            for (pt, st) in p_tx.iter().zip(s_tx.iter()) {
                assert_eq!(pt.transaktions_id, st.transaktions_id);
            }
        }
    }

    #[test]
    fn test_convert_batch_single() {
        let inputs: Vec<&[u8]> = vec![MSG1];
        let results = convert_batch(&inputs, FormatVersion::FV2504);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }
}
