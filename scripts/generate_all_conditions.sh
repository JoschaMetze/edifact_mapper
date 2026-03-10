#!/usr/bin/env bash
# Generate AHB condition evaluators for all message types and format versions.
#
# Usage:
#   ./scripts/generate_all_conditions.sh              # Generate all
#   ./scripts/generate_all_conditions.sh --dry-run     # Preview only
#   ./scripts/generate_all_conditions.sh --force        # Force regenerate
#
# Prerequisites: `claude` CLI must be available in PATH.

set -euo pipefail

EXTRA_ARGS="${*}"
OUTPUT_BASE="crates/automapper-validation/src/generated"

# ── Message type configurations ──
# Format: MSG_TYPE|VARIANT|AHB_FV2504|MIG_FV2504|AHB_FV2510|MIG_FV2510|AHB_FV2604|MIG_FV2604|ALIAS_PATTERN
#
# ALIAS_PATTERN: which format versions share identical conditions
#   "all_same"     = FV2504=FV2510=FV2604 (generate FV2504 only, alias others)
#   "04_differs"   = FV2504≠FV2510, FV2510=FV2604 (generate FV2504+FV2510, alias FV2604→FV2510)
#   "04_differs"   = same as above but FV2604 differs from FV2510 too (generate all 3)
#   "all_differ"   = all different (generate all 3)

declare -A CONFIGS

# --- UTILMD Strom ---
CONFIGS[UTILMD_Strom]="UTILMD|Strom|all_same"

# --- UTILMD Gas ---
CONFIGS[UTILMD_Gas]="UTILMD|Gas|all_same"

# --- Types identical across all 3 FVs ---
CONFIGS[APERAK]="APERAK||all_same"
CONFIGS[CONTRL]="CONTRL||all_same"
CONFIGS[ORDCHG]="ORDCHG||all_same"
CONFIGS[UTILTS]="UTILTS||all_same"

# --- FV2504 differs, FV2510=FV2604 ---
CONFIGS[IFTSTA]="IFTSTA||04_differs"
CONFIGS[INVOIC]="INVOIC||04_differs"
CONFIGS[ORDRSP]="ORDRSP||04_differs"
CONFIGS[PRICAT]="PRICAT||04_differs"
CONFIGS[QUOTES]="QUOTES||04_differs"
CONFIGS[REQOTE]="REQOTE||04_differs"

# --- All differ or FV2510≠FV2604 ---
CONFIGS[COMDIS]="COMDIS||all_differ"
CONFIGS[MSCONS]="MSCONS||all_differ"
CONFIGS[ORDERS]="ORDERS||all_differ"
CONFIGS[PARTIN]="PARTIN||all_differ"
CONFIGS[REMADV]="REMADV||all_differ"

# --- INSRPT: only FV2510+ (no FV2504) ---
CONFIGS[INSRPT]="INSRPT||fv2510_only"

# ── AHB/MIG file lookup ──
find_ahb() {
    local fv="$1" msg="$2" variant="$3"
    local pattern
    if [ -n "$variant" ]; then
        pattern="xml-migs-and-ahbs/${fv}/${msg}_AHB_${variant}*.xml"
    else
        pattern="xml-migs-and-ahbs/${fv}/${msg}_AHB*.xml"
    fi
    ls $pattern 2>/dev/null | head -1
}

find_mig() {
    local fv="$1" msg="$2" variant="$3"
    local pattern
    if [ -n "$variant" ]; then
        pattern="xml-migs-and-ahbs/${fv}/${msg}_MIG_${variant}*.xml"
    else
        pattern="xml-migs-and-ahbs/${fv}/${msg}_MIG*.xml"
    fi
    ls $pattern 2>/dev/null | head -1
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

    mkdir -p "$output_dir"

    cargo run -p automapper-generator -- generate-conditions \
        --ahb-path "$ahb_path" \
        --output-dir "$output_dir" \
        --format-version "$fv" \
        --message-type "$msg_name" \
        $mig_arg \
        --batch-size 15 \
        --max-concurrent 4 \
        $EXTRA_ARGS
}

# ── Main loop ──
for key in "${!CONFIGS[@]}"; do
    IFS='|' read -r msg_type variant alias_pattern <<< "${CONFIGS[$key]}"

    case "$alias_pattern" in
        all_same)
            # Generate FV2504 only (or FV2510 for INSRPT)
            generate_one "$msg_type" "$variant" "FV2504"
            echo "  ALIAS: FV2510, FV2604 → FV2504 (identical conditions)"
            ;;
        04_differs)
            # Generate FV2504 and FV2510
            generate_one "$msg_type" "$variant" "FV2504"
            generate_one "$msg_type" "$variant" "FV2510"
            echo "  ALIAS: FV2604 → FV2510 (identical conditions)"
            ;;
        all_differ)
            # Generate all 3
            generate_one "$msg_type" "$variant" "FV2504"
            generate_one "$msg_type" "$variant" "FV2510"
            generate_one "$msg_type" "$variant" "FV2604"
            ;;
        fv2510_only)
            # No FV2504, generate FV2510 only
            generate_one "$msg_type" "$variant" "FV2510"
            echo "  ALIAS: FV2604 → FV2510 (identical conditions)"
            ;;
    esac
    echo ""
done

echo "=== Generation complete ==="
echo "Now run: cargo clippy -p automapper-validation -- -D warnings"
