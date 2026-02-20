//! Auto-generated PID 55557 types.
//! Änderung MSB-Abr.-Daten der MaLo
//! Do not edit manually.

use serde::{Deserialize, Serialize};

/// PID 55557: Änderung MSB-Abr.-Daten der MaLo
/// Kommunikation: MSB an NB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid55557 {
    pub bgm: super::super::segments::SegBgm,
    pub dtm: super::super::segments::SegDtm,
    pub unh: super::super::segments::SegUnh,
    pub unt: super::super::segments::SegUnt,
    pub sg2: Vec<super::super::groups::Sg2>,
    pub sg4: Vec<super::super::groups::Sg4>,
}
