#!/usr/bin/env bash
# Generate AHB condition evaluators using Claude CLI.
#
# Automatically seeds from a baseline FV when possible, then runs the
# generator. Seeding copies an existing implementation and marks only
# new/changed conditions for regeneration (saves ~90% of LLM calls).
#
# Usage:
#   ./scripts/generate_all_conditions.sh UTILMD_Strom FV2504   # One type + FV
#   ./scripts/generate_all_conditions.sh UTILMD_Strom           # One type, all relevant FVs
#   ./scripts/generate_all_conditions.sh FV2510                 # All types that generate FV2510
#   ./scripts/generate_all_conditions.sh                        # All types, all FVs
#
# Extra flags after the positional args are forwarded to the CLI:
#   ./scripts/generate_all_conditions.sh INVOIC FV2504 --dry-run
#   ./scripts/generate_all_conditions.sh --force
#   ./scripts/generate_all_conditions.sh FV2510 --incremental
#
# Prerequisites: `claude` CLI must be available in PATH.
#   Do NOT run from inside a Claude Code session (nested sessions blocked).

set -euo pipefail

OUTPUT_BASE="crates/automapper-validation/src/generated"

# ── Parse positional args ──
TARGET_TYPE=""
TARGET_FV=""
EXTRA_ARGS=""

for arg in "$@"; do
    case "$arg" in
        --*) EXTRA_ARGS="$EXTRA_ARGS $arg" ;;
        FV*)
            TARGET_FV="$arg"
            ;;
        *)
            if [ -z "$TARGET_TYPE" ]; then
                TARGET_TYPE="$arg"
            else
                echo "Error: unexpected argument '$arg'" >&2; exit 1
            fi
            ;;
    esac
done

# ── Message type configurations ──
# Format: MSG_TYPE|VARIANT|ALIAS_PATTERN
# ALIAS_PATTERN:
#   all_same    = FV2504=FV2510=FV2604 → generate FV2504 only, alias others
#   04_differs  = FV2504≠FV2510, FV2510=FV2604 → generate FV2504+FV2510
#   10_differs  = FV2504=FV2510, FV2604 differs → generate FV2504+FV2604
#   all_differ  = all 3 differ → generate all 3
#   fv2510_only = no FV2504 AHB → generate FV2510 only

declare -A CONFIGS
CONFIGS[UTILMD_Strom]="UTILMD|Strom|04_differs"
CONFIGS[UTILMD_Gas]="UTILMD|Gas|10_differs"
CONFIGS[APERAK]="APERAK||all_same"
CONFIGS[CONTRL]="CONTRL||all_same"
CONFIGS[ORDCHG]="ORDCHG||all_same"
CONFIGS[UTILTS]="UTILTS||all_same"
CONFIGS[IFTSTA]="IFTSTA||04_differs"
CONFIGS[INVOIC]="INVOIC||04_differs"
CONFIGS[ORDRSP]="ORDRSP||04_differs"
CONFIGS[PRICAT]="PRICAT||04_differs"
CONFIGS[QUOTES]="QUOTES||04_differs"
CONFIGS[REQOTE]="REQOTE||04_differs"
CONFIGS[COMDIS]="COMDIS||all_differ"
CONFIGS[MSCONS]="MSCONS||all_differ"
CONFIGS[ORDERS]="ORDERS||all_differ"
CONFIGS[PARTIN]="PARTIN||all_differ"
CONFIGS[REMADV]="REMADV||all_differ"
CONFIGS[INSRPT]="INSRPT||fv2510_only"

# ── Seeding: baseline FV for each (type, target_fv) pair ──
# Derived from alias patterns above. Empty = no baseline (generate from scratch).
declare -A SEED_BASELINE
# 04_differs: FV2510 ← FV2504
SEED_BASELINE[UTILMD_Strom:FV2510]="FV2504"
SEED_BASELINE[IFTSTA:FV2510]="FV2504"
SEED_BASELINE[INVOIC:FV2510]="FV2504"
SEED_BASELINE[ORDRSP:FV2510]="FV2504"
SEED_BASELINE[PRICAT:FV2510]="FV2504"
SEED_BASELINE[QUOTES:FV2510]="FV2504"
SEED_BASELINE[REQOTE:FV2510]="FV2504"
# 10_differs: FV2604 ← FV2504
SEED_BASELINE[UTILMD_Gas:FV2604]="FV2504"
# all_differ: FV2510 ← FV2504, FV2604 ← FV2510
SEED_BASELINE[COMDIS:FV2510]="FV2504"
SEED_BASELINE[COMDIS:FV2604]="FV2510"
SEED_BASELINE[MSCONS:FV2510]="FV2504"
SEED_BASELINE[MSCONS:FV2604]="FV2510"
SEED_BASELINE[ORDERS:FV2510]="FV2504"
SEED_BASELINE[ORDERS:FV2604]="FV2510"
SEED_BASELINE[PARTIN:FV2510]="FV2504"
SEED_BASELINE[PARTIN:FV2604]="FV2510"
SEED_BASELINE[REMADV:FV2510]="FV2504"
SEED_BASELINE[REMADV:FV2604]="FV2510"

# ── Validate target type ──
if [ -n "$TARGET_TYPE" ] && [ -z "${CONFIGS[$TARGET_TYPE]+x}" ]; then
    echo "Error: unknown message type '$TARGET_TYPE'" >&2
    echo "Valid types: ${!CONFIGS[*]}" >&2
    exit 1
fi

# ── AHB/MIG file lookup ──
find_ahb() {
    local fv="$1" msg="$2" variant="$3"
    if [ -n "$variant" ]; then
        ls xml-migs-and-ahbs/${fv}/${msg}_AHB_${variant}*.xml 2>/dev/null | head -1
    else
        ls xml-migs-and-ahbs/${fv}/${msg}_AHB*.xml 2>/dev/null | head -1
    fi
}

find_mig() {
    local fv="$1" msg="$2" variant="$3"
    if [ -n "$variant" ]; then
        ls xml-migs-and-ahbs/${fv}/${msg}_MIG_${variant}*.xml 2>/dev/null | head -1
    else
        ls xml-migs-and-ahbs/${fv}/${msg}_MIG*.xml 2>/dev/null | head -1
    fi
}

# ── Seed from baseline if needed ──
# Seeds the target FV from a baseline FV by copying the .rs file and building
# metadata that marks only changed/new conditions for regeneration.
maybe_seed() {
    local msg_name="$1" msg_type="$2" variant="$3" fv="$4"

    local seed_key="${msg_name}:${fv}"
    local baseline="${SEED_BASELINE[$seed_key]:-}"
    if [ -z "$baseline" ]; then
        return  # No baseline configured for this (type, fv) pair
    fi

    local file_stem=$(echo "$msg_name" | tr '[:upper:]' '[:lower:]')
    local fv_lower=$(echo "$fv" | tr '[:upper:]' '[:lower:]')
    local baseline_lower=$(echo "$baseline" | tr '[:upper:]' '[:lower:]')

    local dst_meta="${OUTPUT_BASE}/${fv_lower}/${file_stem}_condition_evaluator_${fv_lower}.conditions.json"
    local src_meta="${OUTPUT_BASE}/${baseline_lower}/${file_stem}_condition_evaluator_${baseline_lower}.conditions.json"
    local src_rs="${OUTPUT_BASE}/${baseline_lower}/${file_stem}_conditions_${baseline_lower}.rs"
    local dst_rs="${OUTPUT_BASE}/${fv_lower}/${file_stem}_conditions_${fv_lower}.rs"

    # Skip if already seeded/generated
    if [ -f "$dst_meta" ]; then
        return
    fi

    # Need baseline metadata to seed
    if [ ! -f "$src_meta" ] || [ ! -f "$src_rs" ]; then
        echo "  NOTE: Cannot seed from ${baseline} (no metadata). Will generate from scratch."
        return
    fi

    # Resolve AHB XMLs
    local src_ahb=$(find_ahb "$baseline" "$msg_type" "$variant")
    local dst_ahb=$(find_ahb "$fv" "$msg_type" "$variant")
    if [ -z "$src_ahb" ] || [ -z "$dst_ahb" ]; then
        echo "  NOTE: Cannot seed (missing AHB XML). Will generate from scratch."
        return
    fi

    # PascalCase struct prefix
    local struct_prefix=$(echo "$msg_name" | sed -E 's/_(.)/\U\1/g; s/^(.)/\U\1/')
    local src_struct="${struct_prefix}ConditionEvaluator${baseline}"
    local dst_struct="${struct_prefix}ConditionEvaluator${fv}"

    echo "  Seeding from ${baseline}..."

    # Step 1: Copy .rs and rename
    mkdir -p "${OUTPUT_BASE}/${fv_lower}"
    cp "$src_rs" "$dst_rs"
    sed -i "s/${src_struct}/${dst_struct}/g" "$dst_rs"
    sed -i "s/${baseline}/${fv}/g" "$dst_rs"
    sed -i "s|$(basename "$src_ahb")|$(basename "$dst_ahb") (seeded from ${baseline})|" "$dst_rs"

    # Step 2: Compare AHBs and build metadata (Python)
    python3 - "$src_meta" "$dst_meta" "$dst_rs" "$src_ahb" "$dst_ahb" "$baseline" "$fv" << 'PYEOF'
import json, hashlib, re, sys
import xml.etree.ElementTree as ET

src_meta_path, dst_meta_path, dst_rs_path, src_ahb, dst_ahb, src_fv, dst_fv = sys.argv[1:8]

def compute_hash(text):
    return hashlib.sha256(text.encode()).hexdigest()[:8]

def extract_ahb_conditions(ahb_path):
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

with open(src_meta_path) as f:
    src_meta = json.load(f)

conds_src = extract_ahb_conditions(src_ahb)
conds_dst = extract_ahb_conditions(dst_ahb)

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
print(f"    Removed: {len(removed)}, Added: {len(added)}, Changed: {len(changed_desc)}, Unchanged: {len(unchanged)}")

# Build target metadata
dst_conditions = {}

for cid in unchanged:
    src_entry = src_meta['conditions'].get(cid, {})
    dst_conditions[cid] = {
        'confidence': src_entry.get('confidence', 'high'),
        'reasoning': src_entry.get('reasoning', f'[SEEDED] From {src_fv}'),
        'description_hash': compute_hash(conds_dst[cid]),
        'is_external': src_entry.get('is_external', False),
    }

for cid in changed_desc:
    src_entry = src_meta['conditions'].get(cid, {})
    dst_conditions[cid] = {
        'confidence': 'low',
        'reasoning': f'[SEEDED:changed] Description differs from {src_fv}',
        'description_hash': '00000000',
        'is_external': src_entry.get('is_external', False),
    }

# Remove deleted conditions from .rs
if removed:
    with open(dst_rs_path) as f:
        rs_content = f.read()
    for cid in sorted(removed, key=sort_key):
        pattern = (
            rf'(?:\n\s*///[^\n]*\[{cid}\][^\n]*\n(?:\s*///[^\n]*\n)*)?'
            rf'\s*fn evaluate_{cid}\(&self,[^\n]*\n'
            rf'(?:.*?\n)*?\s*\}}\n'
        )
        match = re.search(pattern, rs_content)
        if match:
            rs_content = rs_content[:match.start()] + '\n' + rs_content[match.end():]
        arm_pattern = rf'\s*{cid} => self\.evaluate_{cid}\(ctx\),?\n'
        rs_content = re.sub(arm_pattern, '', rs_content)
    with open(dst_rs_path, 'w') as f:
        f.write(rs_content)

dst_meta_obj = {
    'generated_at': '2026-03-11T00:00:00Z',
    'ahb_file': dst_ahb.split('/')[-1],
    'format_version': dst_fv,
    'conditions': dst_conditions,
}
with open(dst_meta_path, 'w') as f:
    json.dump(dst_meta_obj, f, indent=2, ensure_ascii=False)

total_regen = len(added) + len(changed_desc)
print(f"    Will regenerate: {len(added)} new + {len(changed_desc)} changed = {total_regen} (preserved: {len(unchanged)})")
PYEOF
}

# ── Generate for one message type + format version ──
generate_one() {
    local msg_type="$1" variant="$2" fv="$3"
    local fv_lower=$(echo "$fv" | tr '[:upper:]' '[:lower:]')
    local output_dir="${OUTPUT_BASE}/${fv_lower}"

    local ahb_path=$(find_ahb "$fv" "$msg_type" "$variant")
    local mig_path=$(find_mig "$fv" "$msg_type" "$variant")

    if [ -z "$ahb_path" ]; then
        echo "SKIP: No AHB found for ${msg_type} ${variant} ${fv}"
        return
    fi

    local mig_arg=""
    if [ -n "$mig_path" ]; then
        mig_arg="--mig-path ${mig_path}"
    fi

    local msg_name="${msg_type}"
    if [ -n "$variant" ]; then
        msg_name="${msg_type}_${variant}"
    fi

    echo "=== Generating ${msg_name} ${fv} ==="
    echo "  AHB: ${ahb_path}"
    echo "  MIG: ${mig_path:-none}"

    # Auto-seed from baseline if available
    maybe_seed "$msg_name" "$msg_type" "$variant" "$fv"

    mkdir -p "$output_dir"

    cargo run -p automapper-generator -- generate-conditions \
        --ahb-path "$ahb_path" \
        --output-dir "$output_dir" \
        --format-version "$fv" \
        --message-type "$msg_name" \
        $mig_arg \
        --batch-size 5 \
        --max-concurrent 4 \
        $EXTRA_ARGS
}

# ── Determine which FVs to generate for a given config ──
fvs_for_config() {
    local alias_pattern="$1"
    case "$alias_pattern" in
        all_same)    echo "FV2504" ;;
        04_differs)  echo "FV2504 FV2510" ;;
        10_differs)  echo "FV2504 FV2604" ;;
        all_differ)  echo "FV2504 FV2510 FV2604" ;;
        fv2510_only) echo "FV2510" ;;
    esac
}

# ── Main ──
KEYS=()
if [ -n "$TARGET_TYPE" ]; then
    KEYS=("$TARGET_TYPE")
else
    KEYS=(${!CONFIGS[@]})
fi

for key in "${KEYS[@]}"; do
    IFS='|' read -r msg_type variant alias_pattern <<< "${CONFIGS[$key]}"
    fvs=$(fvs_for_config "$alias_pattern")

    if [ -n "$TARGET_FV" ]; then
        # User requested specific FV — check it's valid for this type
        if echo "$fvs" | grep -qw "$TARGET_FV"; then
            generate_one "$msg_type" "$variant" "$TARGET_FV"
        else
            echo "SKIP: $key does not generate $TARGET_FV (generates: $fvs)"
        fi
    else
        for fv in $fvs; do
            generate_one "$msg_type" "$variant" "$fv"
        done
    fi
    echo ""
done

echo "=== Generation complete ==="
echo "Now run: cargo clippy -p automapper-validation -- -D warnings"
