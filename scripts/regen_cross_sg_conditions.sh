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

# Target specific condition IDs that benefit from the new cross-SG API.
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

# Extract descriptions from the .rs file (doc comments before evaluate_ functions).
# These are used to compute correct description hashes so the generator
# doesn't mark non-target conditions as "stale".
desc_pattern = re.compile(r'///?\s*\[(\d+)\]\s*(.*?)(?=\n\s*fn evaluate_)', re.DOTALL)
rs_descriptions = {}
for m in desc_pattern.finditer(rs_content):
    cid = m.group(1)
    desc = m.group(2).strip()
    # Clean up multi-line doc comments
    desc = re.sub(r'\n\s*///?', ' ', desc).strip()
    rs_descriptions[cid] = desc

def compute_hash(text):
    """Match Rust's compute_description_hash: SHA-256, first 4 bytes as hex."""
    return hashlib.sha256(text.encode()).hexdigest()[:8]

# Force ALL conditions to "high" first (prevents regeneration),
# then selectively downgrade our targets to "low".
all_rs_ids = set(re.findall(r'fn evaluate_(\d+)\(', rs_content))

for cid in all_rs_ids:
    # Compute the real description hash if we have the description
    desc_hash = compute_hash(rs_descriptions[cid]) if cid in rs_descriptions else "00000000"

    if cid not in conds:
        conds[cid] = {
            "confidence": "high",
            "reasoning": "[PRESERVED] Existing implementation",
            "description_hash": desc_hash,
            "is_external": False,
        }
    elif cid not in TARGET_IDS:
        # Force non-targets to high AND update their hash to prevent stale detection
        conds[cid]['confidence'] = 'high'
        if desc_hash != "00000000":
            conds[cid]['description_hash'] = desc_hash

changed = 0
for cid in TARGET_IDS:
    if cid in conds:
        old_conf = conds[cid]['confidence']
        conds[cid]['confidence'] = 'low'
        conds[cid]['reasoning'] = f"[REGEN:{old_conf}] Targeted for cross-SG API regeneration"
        # Use a dummy hash that won't match — forces regeneration
        conds[cid]['description_hash'] = '00000000'
        changed += 1
    else:
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

print(f"  Generator will regenerate ONLY: {by_conf.get('low', 0)} conditions")
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
    --batch-size 5 \
    --max-concurrent 2 \
    --incremental \
    "$@"

echo ""
echo "=== Step 3: Auto-fix common clippy issues ==="
python3 << 'PYEOF2'
import re

path = "crates/automapper-validation/src/generated/fv2504/utilmd_strom_conditions_fv2504.rs"
with open(path) as f:
    content = f.read()

# 1. Fix unused `ctx` → `_ctx` for functions that don't use ctx in their body
lines = content.split('\n')
result = []
i = 0
while i < len(lines):
    line = lines[i]
    m = re.match(r'(\s+fn evaluate_\d+\(&self,\s+)ctx(:\s+&EvaluationContext\)\s+->\s+ConditionResult\s+\{)', line)
    if m:
        body_lines = []
        j = i + 1
        brace_depth = 1
        while j < len(lines) and brace_depth > 0:
            for ch in lines[j]:
                if ch == '{': brace_depth += 1
                elif ch == '}': brace_depth -= 1
            if brace_depth > 0:
                body_lines.append(lines[j])
            j += 1
        body = '\n'.join(body_lines)
        if 'ctx' not in body:
            line = m.group(1) + '_ctx' + m.group(2)
    result.append(line)
    i += 1
content = '\n'.join(result)

# 2. Fix ctx.navigator() → ctx.navigator (field, not method)
content = content.replace('ctx.navigator()', 'ctx.navigator')

# 3. Fix .get(0) → .first()
content = content.replace('.get(0)', '.first()')

# 3. Fix .map(|v| v.clone()) → .cloned()
content = re.sub(r'\.map\(\|v\| v\.clone\(\)\)', '.cloned()', content)

# 4. Fix type annotations: .and_then(|e| e.first()) needs Vec<String> annotation
content = re.sub(
    r'\.and_then\(\|e\| e\.first\(\)\)',
    '.and_then(|e: &Vec<String>| e.first())',
    content
)

# 5. Fix type annotations: .is_some_and(|v| ...) needs &String annotation
content = re.sub(
    r'\.is_some_and\(\|v\| ',
    '.is_some_and(|v: &String| ',
    content
)

# 6. Fix for_kv_map: for (_x, y) in &map → for y in map.values()
content = re.sub(r'for \(_\w+, (\w+)\) in &(\w+)', r'for \1 in \2.values()', content)

with open(path, 'w') as f:
    f.write(content)

print("  Auto-fixed: unused ctx, navigator(), .get(0), type annotations, .map(|v| v.clone()), for_kv_map")
PYEOF2

echo ""
echo "=== Step 4: Verify ==="
cargo clippy -p automapper-validation -- -D warnings
cargo test -p automapper-validation --lib

echo ""
echo "=== Done ==="
echo "Review the changes:"
echo "  git diff crates/automapper-validation/src/generated/fv2504/utilmd_strom_conditions_fv2504.rs"
