#!/usr/bin/env bash
# Generate AHB condition evaluators using Claude CLI.
#
# Usage:
#   ./scripts/generate_all_conditions.sh UTILMD_Strom FV2504   # One type + FV
#   ./scripts/generate_all_conditions.sh UTILMD_Strom           # One type, all relevant FVs
#   ./scripts/generate_all_conditions.sh                        # All types, all FVs
#
# Extra flags after the positional args are forwarded to the CLI:
#   ./scripts/generate_all_conditions.sh INVOIC FV2504 --dry-run
#   ./scripts/generate_all_conditions.sh --force
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
            if [ -z "$TARGET_TYPE" ]; then
                echo "Error: format version '$arg' given before message type" >&2; exit 1
            fi
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
#   all_same   = FV2504=FV2510=FV2604 → generate FV2504 only, alias others
#   04_differs = FV2504≠FV2510, FV2510=FV2604 → generate FV2504+FV2510
#   all_differ = all 3 differ → generate all 3
#   fv2510_only = no FV2504 AHB → generate FV2510 only

declare -A CONFIGS
CONFIGS[UTILMD_Strom]="UTILMD|Strom|all_same"
CONFIGS[UTILMD_Gas]="UTILMD|Gas|all_same"
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

# ── Determine which FVs to generate for a given config ──
fvs_for_config() {
    local alias_pattern="$1"
    case "$alias_pattern" in
        all_same)    echo "FV2504" ;;
        04_differs)  echo "FV2504 FV2510" ;;
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
