/// This test generates the actual mig-types code. Run it explicitly when re-generating.
/// It writes to `crates/mig-types/src/generated/` which is committed.
use automapper_generator::codegen::mig_type_gen;
use std::path::Path;

#[test]
#[ignore] // Only run explicitly: cargo test -p automapper-generator generate_real -- --ignored
fn generate_real_utilmd_types() {
    let output_dir = Path::new("../../crates/mig-types/src/generated");
    mig_type_gen::generate_mig_types(
        Path::new(
            "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
        ),
        "UTILMD",
        Some("Strom"),
        "FV2504",
        output_dir,
    )
    .expect("Failed to generate MIG types");

    println!("Generated UTILMD types to {:?}", output_dir);
}
