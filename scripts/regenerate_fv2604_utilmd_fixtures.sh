#!/bin/bash
# Regenerate all FV2604 UTILMD enhanced fixtures (Strom + Gas).
#
# Uses the fixture generator CLI with --enhance to produce roundtrip-compatible
# fixtures that go through the full BO4E mapping pipeline.
#
# Usage: ./scripts/regenerate_fv2604_utilmd_fixtures.sh [strom|gas]
# Without args: regenerate both. With arg: regenerate only that variant.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

FV="FV2604"
FV_LOWER="fv2604"
MIG_DIR="xml-migs-and-ahbs/$FV"
SCHEMA_DIR="crates/mig-types/src/generated/$FV_LOWER/utilmd/pids"
FIXTURE_BASE="example_market_communication_bo4e_transactions/UTILMD/$FV"

FILTER="${1:-}"
total_generated=0
total_failed=0

TX_GROUP="SG4"

# ── Strom ──
if [[ -z "$FILTER" || "$FILTER" == "strom" ]]; then
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "UTILMD Strom ($FV)"
    echo "═══════════════════════════════════════════════════════════"

    STROM_MIG="UTILMD_MIG_Strom_S2_1_außerordentlicheg_20251211.xml"
    STROM_AHB="UTILMD_AHB_Strom_2_1_außerordentliche_20251211.xml"
    OUTPUT_DIR="$FIXTURE_BASE/generated_strom"

    for schema_file in "$SCHEMA_DIR"/pid_5*_schema.json; do
        [[ -f "$schema_file" ]] || continue
        pid=$(basename "$schema_file" | sed 's/pid_//;s/_schema\.json//')

        output_file="$OUTPUT_DIR/${pid}.edi"
        echo -n "  PID $pid: "

        if cargo run -q -p automapper-generator -- generate-fixture \
            --pid-schema "$schema_file" \
            --output "$output_file" \
            --enhance \
            --mig-xml "$MIG_DIR/$STROM_MIG" \
            --ahb-xml "$MIG_DIR/$STROM_AHB" \
            --message-type "UTILMD" \
            --variant "Strom" \
            --format-version "$FV" \
            --tx-group "$TX_GROUP" \
            2>/tmp/fixture_gen_utilmd_$pid.log; then
            seg_count=$(grep -o "'" "$output_file" 2>/dev/null | wc -l)
            echo "OK ($seg_count segs)"
            total_generated=$((total_generated + 1))
        else
            echo "FAILED"
            tail -3 /tmp/fixture_gen_utilmd_$pid.log 2>/dev/null
            total_failed=$((total_failed + 1))
        fi
    done
fi

# ── Gas ──
if [[ -z "$FILTER" || "$FILTER" == "gas" ]]; then
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "UTILMD Gas ($FV)"
    echo "═══════════════════════════════════════════════════════════"

    GAS_MIG="UTILMD_MIG_Gas_G1_1_außerordentliche_20251211.xml"
    GAS_AHB="UTILMD_AHB_Gas_1_1_außerordentliche_20251211.xml"
    OUTPUT_DIR="$FIXTURE_BASE/generated_gas"

    for schema_file in "$SCHEMA_DIR"/pid_4*_schema.json; do
        [[ -f "$schema_file" ]] || continue
        pid=$(basename "$schema_file" | sed 's/pid_//;s/_schema\.json//')

        output_file="$OUTPUT_DIR/${pid}.edi"
        echo -n "  PID $pid: "

        if cargo run -q -p automapper-generator -- generate-fixture \
            --pid-schema "$schema_file" \
            --output "$output_file" \
            --enhance \
            --mig-xml "$MIG_DIR/$GAS_MIG" \
            --ahb-xml "$MIG_DIR/$GAS_AHB" \
            --message-type "UTILMD" \
            --variant "Gas" \
            --format-version "$FV" \
            --tx-group "$TX_GROUP" \
            2>/tmp/fixture_gen_utilmd_$pid.log; then
            seg_count=$(grep -o "'" "$output_file" 2>/dev/null | wc -l)
            echo "OK ($seg_count segs)"
            total_generated=$((total_generated + 1))
        else
            echo "FAILED"
            tail -3 /tmp/fixture_gen_utilmd_$pid.log 2>/dev/null
            total_failed=$((total_failed + 1))
        fi
    done
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "Summary: $total_generated enhanced, $total_failed failed"
echo "═══════════════════════════════════════════════════════════"
