# Code Enrichment for Companion Fields

**Date:** 2026-02-24
**Status:** Approved

## Problem

Companion fields store raw EDIFACT codes (e.g., `"Z15"`, `"Z66"`) which are not human-readable. Consumers of the BO4E JSON output have no way to know what these codes mean without referencing external documentation.

## Solution

Automatically enrich companion field values with human-readable meanings from the PID schema JSON. The PID schema already contains code descriptions (e.g., `{"value": "Z15", "name": "Ja"}`).

### Forward Mapping (EDIFACT → BO4E)

Code-type companion fields produce structured objects instead of plain strings:

```json
{
  "MarktlokationEdifact": {
    "haushaltskunde": {
      "code": "Z15",
      "meaning": "Ja"
    },
    "merkmalCode": {
      "code": "Z18",
      "meaning": "Netzlokation"
    }
  }
}
```

Data-type companion fields remain plain strings (e.g., `"zugeordneterMp": "DE0001234"`).

### Reverse Mapping (BO4E → EDIFACT)

Accepts both formats:
- Object: `{"code": "Z15", "meaning": "Ja"}` → extracts `"Z15"`
- Plain string: `"Z15"` → uses as-is (backward compatibility)

### Scope

- Only `[companion_fields]` — regular `[fields]` are not enriched
- No TOML mapping file changes required — auto-detected from PID schema

## Design

### CodeLookup Structure

```rust
/// Maps (source_path, segment_tag, element_index, component_index) → code_value→meaning
type CodeLookup = HashMap<(String, String, usize, usize), BTreeMap<String, String>>;
```

Built at load time from PID schema JSON (`crates/mig-types/src/generated/.../pid_NNNNN_schema.json`).

### MappingEngine Changes

1. **New field:** `code_lookup: Option<CodeLookup>` on `MappingEngine`
2. **New constructor:** `with_code_lookup(schema_path)` or `load_code_lookup(schema_path)` method
3. **Forward:** `extract_companion_fields()` checks lookup; if code field, emits `{code, meaning}` object; unknown codes get `meaning: null`
4. **Reverse:** companion field extraction checks if JSON value is object with `"code"` key; if so, uses `.code`; if plain string, uses directly

### Schema Parsing

PID schema JSON structure for code fields:

```json
{
  "id": "CCI",
  "elements": [{
    "index": 2,
    "components": [{
      "sub_index": 0,
      "type": "code",
      "codes": [
        {"value": "Z15", "name": "Ja"},
        {"value": "Z16", "name": "Nein"}
      ]
    }]
  }]
}
```

Parse by walking the group hierarchy, collecting all code-type components with their group path, segment tag, element index, and component sub-index.

### Backward Compatibility

- When no schema is loaded (`code_lookup = None`), behavior is identical to today
- Reverse mapping accepts both old (string) and new (object) formats
- Existing TOML files work without modification
