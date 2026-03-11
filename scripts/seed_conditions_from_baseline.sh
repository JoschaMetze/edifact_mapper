#!/usr/bin/env bash
# Seed condition evaluators for a new format version from an existing baseline.
#
# When a new FV (e.g., FV2510) shares most conditions with an existing FV
# (e.g., FV2504), this script copies the implementation and adjusts metadata
# so that --incremental generation only regenerates the conditions that
# actually differ (new, changed, or removed).
#
# Usage:
#   ./scripts/seed_conditions_from_baseline.sh <MSG_TYPE> <SRC_FV> <DST_FV>
#
# Examples:
#   ./scripts/seed_conditions_from_baseline.sh UTILMD_Strom FV2504 FV2510
#   ./scripts/seed_conditions_from_baseline.sh UTILMD_Gas FV2504 FV2604
#
# After seeding, run the generator externally:
#   ./scripts/generate_all_conditions.sh UTILMD_Strom FV2510 --incremental

set -euo pipefail

if [ $# -lt 3 ]; then
    echo "Usage: $0 <MSG_TYPE> <SRC_FV> <DST_FV>"
    echo "  MSG_TYPE: e.g., UTILMD_Strom, UTILMD_Gas, MSCONS, ORDERS, ..."
    echo "  SRC_FV:   source format version (e.g., FV2504)"
    echo "  DST_FV:   target format version (e.g., FV2510)"
    exit 1
fi

MSG_TYPE="$1"
SRC_FV="$2"
DST_FV="$3"

# ── Derive file names ──
# MSG_TYPE "UTILMD_Strom" → file stem "utilmd_strom", struct prefix "UtilmdStrom"
FILE_STEM=$(echo "$MSG_TYPE" | tr '[:upper:]' '[:lower:]')
# PascalCase: split on _ , capitalize first letter of each part
STRUCT_PREFIX=$(echo "$MSG_TYPE" | sed -E 's/_(.)/\U\1/g; s/^(.)/\U\1/')

SRC_FV_LOWER=$(echo "$SRC_FV" | tr '[:upper:]' '[:lower:]')
DST_FV_LOWER=$(echo "$DST_FV" | tr '[:upper:]' '[:lower:]')

BASE_DIR="crates/automapper-validation/src/generated"
SRC_DIR="${BASE_DIR}/${SRC_FV_LOWER}"
DST_DIR="${BASE_DIR}/${DST_FV_LOWER}"

SRC_RS="${SRC_DIR}/${FILE_STEM}_conditions_${SRC_FV_LOWER}.rs"
DST_RS="${DST_DIR}/${FILE_STEM}_conditions_${DST_FV_LOWER}.rs"
SRC_META="${SRC_DIR}/${FILE_STEM}_condition_evaluator_${SRC_FV_LOWER}.conditions.json"
DST_META="${DST_DIR}/${FILE_STEM}_condition_evaluator_${DST_FV_LOWER}.conditions.json"

SRC_STRUCT="${STRUCT_PREFIX}ConditionEvaluator${SRC_FV}"
DST_STRUCT="${STRUCT_PREFIX}ConditionEvaluator${DST_FV}"

# ── Validate inputs ──
if [ ! -f "$SRC_RS" ]; then
    echo "ERROR: Source .rs not found: $SRC_RS" >&2
    exit 1
fi
if [ ! -f "$SRC_META" ]; then
    echo "ERROR: Source metadata not found: $SRC_META" >&2
    exit 1
fi

# ── Resolve AHB XML files ──
# Split MSG_TYPE into message + variant (e.g., UTILMD_Strom → UTILMD, Strom)
MSG=$(echo "$MSG_TYPE" | cut -d_ -f1)
VARIANT=$(echo "$MSG_TYPE" | cut -s -d_ -f2-)

find_ahb() {
    local fv="$1"
    if [ -n "$VARIANT" ]; then
        ls xml-migs-and-ahbs/${fv}/${MSG}_AHB_${VARIANT}*.xml 2>/dev/null | head -1
    else
        ls xml-migs-and-ahbs/${fv}/${MSG}_AHB*.xml 2>/dev/null | head -1
    fi
}

SRC_AHB=$(find_ahb "$SRC_FV")
DST_AHB=$(find_ahb "$DST_FV")

if [ -z "$SRC_AHB" ]; then
    echo "ERROR: No AHB found for ${MSG_TYPE} ${SRC_FV}" >&2; exit 1
fi
if [ -z "$DST_AHB" ]; then
    echo "ERROR: No AHB found for ${MSG_TYPE} ${DST_FV}" >&2; exit 1
fi

echo "=== Seeding ${MSG_TYPE} conditions: ${SRC_FV} → ${DST_FV} ==="
echo "  Source .rs:   $SRC_RS"
echo "  Target .rs:   $DST_RS"
echo "  Source AHB:   $SRC_AHB"
echo "  Target AHB:   $DST_AHB"

echo ""
echo "=== Step 1: Copy .rs file and rename ==="
mkdir -p "$DST_DIR"
cp "$SRC_RS" "$DST_RS"
sed -i "s/${SRC_STRUCT}/${DST_STRUCT}/g" "$DST_RS"
sed -i "s/${SRC_FV}/${DST_FV}/g" "$DST_RS"
sed -i "s|$(basename "$SRC_AHB")|$(basename "$DST_AHB") (seeded from ${SRC_FV})|" "$DST_RS"
echo "  Copied and renamed: $DST_RS"

echo ""
echo "=== Step 2: Compare AHBs and build metadata ==="
python3 - "$SRC_META" "$DST_META" "$DST_RS" "$SRC_AHB" "$DST_AHB" "$SRC_FV" "$DST_FV" << 'PYEOF'
import json, hashlib, re, sys
import xml.etree.ElementTree as ET

src_meta_path, dst_meta_path, dst_rs_path, src_ahb, dst_ahb, src_fv, dst_fv = sys.argv[1:8]

def compute_hash(text):
    return hashlib.sha256(text.encode()).hexdigest()[:8]

def extract_ahb_conditions(ahb_path):
    """Extract condition ID -> description from AHB XML."""
    tree = ET.parse(ahb_path)
    root = tree.getroot()
    conditions = {}
    for elem in root.iter():
        if elem.tag == 'Bedingung':
            cid = elem.get('Nummer', '').strip('[]')
            desc = (elem.text or '').strip()
            if cid and desc:
                conditions[cid] = desc
    return conditions

# Load source metadata
with open(src_meta_path) as f:
    src_meta = json.load(f)

# Extract conditions from both AHBs
print(f"  Parsing {src_fv} AHB...")
conds_src = extract_ahb_conditions(src_ahb)
print(f"  {src_fv}: {len(conds_src)} conditions")

print(f"  Parsing {dst_fv} AHB...")
conds_dst = extract_ahb_conditions(dst_ahb)
print(f"  {dst_fv}: {len(conds_dst)} conditions")

# Classify
ids_src = set(conds_src.keys())
ids_dst = set(conds_dst.keys())

removed = ids_src - ids_dst
added = ids_dst - ids_src
shared = ids_src & ids_dst

changed_desc = set()
unchanged = set()
for cid in shared:
    if conds_src[cid] != conds_dst[cid]:
        changed_desc.add(cid)
    else:
        unchanged.add(cid)

sort_key = lambda x: int(x) if x.isdigit() else 0

print(f"\n  Summary:")
print(f"    Removed:   {len(removed):>3} — {sorted(removed, key=sort_key)}")
print(f"    Added:     {len(added):>3} — {sorted(added, key=sort_key)}")
print(f"    Changed:   {len(changed_desc):>3} — {sorted(changed_desc, key=sort_key)}")
print(f"    Unchanged: {len(unchanged):>3}")

# Build target metadata
dst_conditions = {}

# 1. Unchanged conditions: keep confidence + set correct hash for target AHB
for cid in unchanged:
    src_entry = src_meta['conditions'].get(cid, {})
    dst_conditions[cid] = {
        'confidence': src_entry.get('confidence', 'high'),
        'reasoning': src_entry.get('reasoning', f'[SEEDED] From {src_fv}'),
        'description_hash': compute_hash(conds_dst[cid]),
        'is_external': src_entry.get('is_external', False),
    }

# 2. Changed conditions: LOW confidence + dummy hash → forces regeneration
for cid in changed_desc:
    src_entry = src_meta['conditions'].get(cid, {})
    dst_conditions[cid] = {
        'confidence': 'low',
        'reasoning': f'[SEEDED:changed] Description differs from {src_fv}',
        'description_hash': '00000000',
        'is_external': src_entry.get('is_external', False),
    }

# 3. Added conditions: omit from metadata → generator detects as New

# 4. Removed conditions: remove evaluate_N functions and match arms from .rs
if removed:
    print(f"\n  Removing {len(removed)} deleted conditions from .rs file...")
    with open(dst_rs_path) as f:
        rs_content = f.read()

    for cid in sorted(removed, key=sort_key):
        # Remove function + doc comments above it
        # Match: optional doc comments, then fn evaluate_N(...) { ... }
        pattern = (
            rf'(?:\n\s*///[^\n]*\[{cid}\][^\n]*\n(?:\s*///[^\n]*\n)*)?'
            rf'\s*fn evaluate_{cid}\(&self,[^\n]*\n'
            rf'(?:.*?\n)*?\s*\}}\n'
        )
        match = re.search(pattern, rs_content)
        if match:
            rs_content = rs_content[:match.start()] + '\n' + rs_content[match.end():]
            print(f"    Removed evaluate_{cid}")
        else:
            print(f"    WARNING: evaluate_{cid} not found in .rs")

        # Remove match arm
        arm_pattern = rf'\s*{cid} => self\.evaluate_{cid}\(ctx\),?\n'
        rs_content = re.sub(arm_pattern, '', rs_content)

    with open(dst_rs_path, 'w') as f:
        f.write(rs_content)

# Save metadata
dst_meta = {
    'generated_at': '2026-03-11T00:00:00Z',
    'ahb_file': dst_ahb.split('/')[-1],
    'format_version': dst_fv,
    'conditions': dst_conditions,
}

with open(dst_meta_path, 'w') as f:
    json.dump(dst_meta, f, indent=2, ensure_ascii=False)

total_regen = len(added) + len(changed_desc)
print(f"\n  Metadata written: {dst_meta_path}")
print(f"  Entries: {len(dst_conditions)} (unchanged: {len(unchanged)}, changed: {len(changed_desc)})")
print(f"  Generator will regenerate: {len(added)} new + {len(changed_desc)} changed = {total_regen} conditions")
PYEOF

echo ""
echo "=== Step 3: Verify compilation ==="
cargo check -p automapper-validation

echo ""
echo "=== Done ==="
echo "Now run the generator externally:"
echo "  ./scripts/generate_all_conditions.sh ${MSG_TYPE} ${DST_FV} --incremental"
