#!/bin/bash
# Regenerate all FV2510 enhanced fixtures for non-UTILMD message types.
#
# Uses the fixture generator CLI with --enhance to produce roundtrip-compatible
# fixtures that go through the full BO4E mapping pipeline.
#
# Usage: ./scripts/regenerate_fv2510_fixtures.sh [MESSAGE_TYPE]
# Without args: regenerate all types. With arg: regenerate only that type.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

FV="FV2510"
FV_LOWER="fv2510"
MIG_DIR="xml-migs-and-ahbs/$FV"
SCHEMA_BASE="crates/mig-types/src/generated/$FV_LOWER"
FIXTURE_BASE="example_market_communication_bo4e_transactions"

FILTER="${1:-}"
total_generated=0
total_failed=0
total_skipped=0

# IFTSTA Family A PIDs (MaBiS) use tx_group=SG4
IFTSTA_FAMILY_A="21000 21001 21002 21003 21004 21005"

# Get tx_group for a given message type and PID
get_tx_group() {
    local msg_type="$1"
    local pid="$2"
    case "$msg_type" in
        APERAK|CONTRL|ORDCHG) echo "" ;;
        COMDIS) echo "SG2" ;;
        IFTSTA)
            if echo "$IFTSTA_FAMILY_A" | grep -qw "$pid"; then
                echo "SG4"
            else
                echo "SG14"
            fi
            ;;
        INSRPT) echo "SG3" ;;
        INVOIC) echo "SG26" ;;
        MSCONS) echo "SG5" ;;
        ORDERS) echo "SG29" ;;
        ORDRSP|QUOTES|REQOTE) echo "SG27" ;;
        PARTIN) echo "SG4" ;;
        PRICAT) echo "SG17" ;;
        REMADV|UTILTS) echo "SG5" ;;
        *) echo "SG4" ;;
    esac
}

# Message type configs: TYPE|MIG_XML|AHB_XML
CONFIGS=(
    "APERAK|APERAK_MIG_2_1i_20240619.xml|APERAK_AHB_1_0_20250401.xml"
    "COMDIS|COMDIS_MIG_1_0f__20250401.xml|COMDIS_AHB_1_0g__20250401.xml"
    "CONTRL|CONTRL_MIG_2_0b_außerordentliche_20251211.xml|CONTRL_AHB_1_0_außerordentliche_20251211.xml"
    "IFTSTA|IFTSTA_MIG_2_0g_20250401.xml|IFTSTA_AHB_2_0h_Fehlerkorrektur_20250623.xml"
    "INSRPT|INSRPT_MIG_1_1a_außerordentliche_20240726.xml|INSRPT_AHB_1_1g_außerordentliche_20251211.xml"
    "INVOIC|INVOIC_MIG_2.8e__20250401.xml|INVOIC_AHB_1_0_Fehlerkorrektur_20250623.xml"
    "MSCONS|MSCONS_MIG_2_4c_außerordentliche_20240726.xml|MSCONS_AHB_3_1f_Fehlerkorrektur_20250623.xml"
    "ORDCHG|ORDCHG_MIG_1_1_außerordentliche_20240726.xml|ORDCHG_AHB_1_0a_20241001.xml"
    "ORDERS|ORDERS_MIG_1_4b_20250401.xml|ORDERS_AHB_1_1_Fehlerkorrektur_20250623.xml"
    "ORDRSP|ORDRSP_MIG_1_4a_20250401.xml|ORDRSP_AHB_1_1_Fehlerkorrektur_20250417.xml"
    "PARTIN|PARTIN_MIG_1_0e_20241001.xml|PARTIN_AHB_1_0e_20241001.xml"
    "PRICAT|PRICAT_MIG_2_0e_2025041.xml|PRICAT_AHB_2_0f_Fehlerkorrektur_20251211.xml"
    "QUOTES|QUOTES_MIG_1_3b_20250401.xml|QUOTES_AHB_1_1_20250401.xml"
    "REMADV|REMADV_MIG_2.9d_20250401.xml|REMADV_AHB_1_0__20250401.xml"
    "REQOTE|REQOTE_MIG_1_3c_20250401.xml|REQOTE_AHB_1_1_20250401.xml"
    "UTILTS|UTILTS_MIG_1.1e_Fehlerkorrektur_20241213.xml|UTILTS_AHB_1_0_Fehlerkorrektur_20251211.xml"
)

for config in "${CONFIGS[@]}"; do
    IFS='|' read -r MSG_TYPE MIG_XML AHB_XML <<< "$config"

    # Filter by message type if specified
    if [[ -n "$FILTER" && "$MSG_TYPE" != "$FILTER" ]]; then
        continue
    fi

    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "$MSG_TYPE (FV2510)"
    echo "═══════════════════════════════════════════════════════════"

    MSG_TYPE_LOWER=$(echo "$MSG_TYPE" | tr '[:upper:]' '[:lower:]')
    SCHEMA_DIR="$SCHEMA_BASE/$MSG_TYPE_LOWER/pids"
    OUTPUT_DIR="$FIXTURE_BASE/$MSG_TYPE/$FV/generated"

    # Find all PID schema files
    if [[ ! -d "$SCHEMA_DIR" ]]; then
        echo "  SKIP: schema dir not found: $SCHEMA_DIR"
        continue
    fi

    for schema_file in "$SCHEMA_DIR"/pid_*_schema.json; do
        [[ -f "$schema_file" ]] || continue

        # Extract PID from filename: pid_55001_schema.json → 55001
        pid=$(basename "$schema_file" | sed 's/pid_//;s/_schema\.json//')

        output_file="$OUTPUT_DIR/${pid}.edi"
        tx_group=$(get_tx_group "$MSG_TYPE" "$pid")

        echo -n "  PID $pid (tx=$tx_group): "

        # Build tx-group arg
        if [[ -n "$tx_group" ]]; then
            tx_arg="--tx-group $tx_group"
        else
            tx_arg="--tx-group \"\""
        fi

        if cargo run -q -p automapper-generator -- generate-fixture \
            --pid-schema "$schema_file" \
            --output "$output_file" \
            --enhance \
            --mig-xml "$MIG_DIR/$MIG_XML" \
            --ahb-xml "$MIG_DIR/$AHB_XML" \
            --message-type "$MSG_TYPE" \
            --variant "" \
            --format-version "$FV" \
            --tx-group "$tx_group" \
            2>/tmp/fixture_gen_$pid.log; then
            seg_count=$(grep -o "'" "$output_file" 2>/dev/null | wc -l)
            echo "OK ($seg_count segs)"
            total_generated=$((total_generated + 1))
        else
            # Check if it fell back to unenhanced
            if grep -q "Falling back" /tmp/fixture_gen_$pid.log 2>/dev/null; then
                echo "UNENHANCED (no mapping dirs)"
                total_skipped=$((total_skipped + 1))
            else
                echo "FAILED"
                tail -3 /tmp/fixture_gen_$pid.log 2>/dev/null
                total_failed=$((total_failed + 1))
            fi
        fi
    done
done

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "Summary: $total_generated enhanced, $total_skipped unenhanced, $total_failed failed"
echo "═══════════════════════════════════════════════════════════"
