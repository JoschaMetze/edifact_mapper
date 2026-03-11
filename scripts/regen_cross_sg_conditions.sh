#!/usr/bin/env bash
# Regenerate conditions that benefit from the new parent-child group
# navigation API (filtered_parent_child_has_qualifier,
# any_group_has_qualifier_without, groups_share_qualified_value).
#
# Must be run OUTSIDE a Claude Code session (uses claude --print).
#
# Usage:
#   ./scripts/regen_cross_sg_conditions.sh           # Full run
#   ./scripts/regen_cross_sg_conditions.sh --dry-run  # Parse only, no LLM calls

set -euo pipefail

AHB="xml-migs-and-ahbs/FV2504/UTILMD_AHB_Strom_2_1_Fehlerkorrektur_20250623.xml"
MIG="xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml"
OUTPUT_DIR="crates/automapper-validation/src/generated/fv2504"
METADATA="${OUTPUT_DIR}/utilmd_strom_condition_evaluator_fv2504.conditions.json"

if [ -n "${CLAUDECODE:-}" ]; then
    echo "ERROR: Cannot run inside a Claude Code session (nested sessions blocked)."
    echo "Run this script from a regular terminal."
    exit 1
fi

echo "=== Step 1: Prepare metadata for targeted regeneration ==="
python3 << 'PYEOF'
import json, re, hashlib

metadata_path = "crates/automapper-validation/src/generated/fv2504/utilmd_strom_condition_evaluator_fv2504.conditions.json"
rs_path = "crates/automapper-validation/src/generated/fv2504/utilmd_strom_conditions_fv2504.rs"

with open(metadata_path) as f:
    data = json.load(f)

with open(rs_path) as f:
    rs_content = f.read()

conds = data.get('conditions', {})

# 1. Backfill: add "high" metadata for ALL conditions in the .rs file
#    that have no metadata entry. This prevents them from being regenerated
#    as "New" — we only want to regenerate our specific targets.
all_rs_ids = set(re.findall(r'fn evaluate_(\d+)\(', rs_content))
existing_ids = set(conds.keys())
missing_ids = all_rs_ids - existing_ids

if missing_ids:
    print(f"  Backfilling {len(missing_ids)} conditions with no metadata (marking high to skip)")
    for cid in missing_ids:
        conds[cid] = {
            "confidence": "high",
            "reasoning": "[STUB] No previous generation — preserved as-is",
            "description_hash": "00000000",
            "is_external": False,
        }

# 2. Target specific condition IDs that benefit from the new cross-SG API.
#    These were identified by analyzing condition reasoning for:
#    - Pattern A: parent->child SG8->SG10 navigation
#    - Pattern B: presence + absence in same group instance
#    - Pattern C: cross-group value correlation (Zeitraum-ID matching)
TARGET_IDS = {
    # Pattern A: parent->child (filtered_parent_child_has_qualifier)
    "47", "76", "90", "114", "117", "118", "122", "193", "197", "199",
    "377", "380", "403", "404", "445",
    # Pattern B: presence+absence (any_group_has_qualifier_without)
    "91", "111", "112", "121", "139", "173", "204", "307", "316",
    "378", "444",
    # Pattern C: cross-group value correlation (groups_share_qualified_value)
    "22", "44", "50", "51", "62", "89", "123", "124", "132", "134",
    "135", "149", "175", "196", "202", "229", "266", "306", "327",
    "332", "335", "399",
    # Cardinality conditions using cross-group navigation
    "2004", "2005", "2006", "2007", "2008", "2009", "2011",
}

changed = 0
for cid in TARGET_IDS:
    if cid in conds and conds[cid]['confidence'] != 'low':
        old_conf = conds[cid]['confidence']
        conds[cid]['confidence'] = 'low'
        conds[cid]['reasoning'] = f"[REGEN:{old_conf}] {conds[cid].get('reasoning', '')}"
        changed += 1
    elif cid not in conds:
        print(f"  WARNING: target condition [{cid}] not found in metadata")

by_conf = {}
for m in conds.values():
    c = m.get('confidence', 'unknown')
    by_conf[c] = by_conf.get(c, 0) + 1

print(f"  Targeted conditions: {len(TARGET_IDS)}")
print(f"  Downgraded to low: {changed}")
print(f"  Final distribution: {by_conf}")

with open(metadata_path, 'w') as f:
    json.dump(data, f, indent=2, ensure_ascii=False)

print(f"  Generator will regenerate: {by_conf.get('low', 0)} conditions")
PYEOF

echo ""
echo "=== Step 2: Regenerate with incremental mode ==="
echo "  Only low-confidence conditions will be sent to Claude."
cargo run -p automapper-generator -- generate-conditions \
    --ahb-path "$AHB" \
    --output-dir "$OUTPUT_DIR" \
    --format-version FV2504 \
    --message-type UTILMD_Strom \
    --mig-path "$MIG" \
    --batch-size 15 \
    --max-concurrent 4 \
    --incremental \
    "$@"

echo ""
echo "=== Step 3: Verify ==="
cargo clippy -p automapper-validation -- -D warnings
cargo test -p automapper-validation --lib

echo ""
echo "=== Done ==="
echo "Review the changes:"
echo "  git diff crates/automapper-validation/src/generated/fv2504/utilmd_strom_conditions_fv2504.rs"
