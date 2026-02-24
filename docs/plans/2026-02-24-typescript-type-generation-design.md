# TypeScript Type Generation from PID Schemas & TOML Mappings

**Date:** 2026-02-24
**Status:** Design

## Goal

Generate TypeScript type definitions (`.d.ts` files) from existing PID schema JSONs and TOML mapping files. This gives frontend consumers (the Leptos WASM frontend and external TS/JS clients) compile-time type safety for the BO4E JSON produced by `POST /api/v2/convert`.

## Architecture Overview

A new `generate-typescript` subcommand on `automapper-generator` reads PID schema JSONs and TOML mapping directories, then emits `.d.ts` files into `generated/typescript/`. Two levels of types are produced:

1. **Per-entity interfaces** — one interface per BO4E entity (e.g. `Marktlokation`, `Prozessdaten`), shared across PIDs.
2. **Per-PID response types** — one composition type per PID (e.g. `Pid55001Response`) that picks and composes the relevant entity interfaces.

Generated files are committed following the existing convention for generated code.

## Output Structure

```
generated/typescript/
  common.d.ts                          # CodeField, shared utilities
  FV2504/
    UTILMD_Strom/
      pid_55001.d.ts                   # Entity interfaces + Pid55001Response
      pid_55002.d.ts                   # Entity interfaces + Pid55002Response
      index.d.ts                       # Barrel re-exports
```

### Shared Types (`common.d.ts`)

```typescript
/** Enriched code value returned when enrich_codes=true. */
export interface CodeValue {
  code: string;
  meaning: string | null;
}

/**
 * Fields that are code-type in the PID schema.
 * Plain string when enrich_codes=false, CodeValue object when enrich_codes=true.
 */
export type CodeField = string | CodeValue;
```

### Per-Entity Interfaces

Derived from TOML `[fields]` targets. All fields are optional since EDIFACT messages may omit any field.

```typescript
export interface Marktlokation {
  marktlokationsId?: string;
}

export interface MarktlokationEdifact {
  haushaltskunde?: CodeField;
  bilanzierungsmethode?: CodeField;
}

export interface Prozessdaten {
  vorgangId?: string;
  gueltigAb?: string;
  gueltigBis?: string;
  transaktionsgrund?: string;
  transaktionsgrundErgaenzung?: string;
  transaktionsgrundErgaenzungBefristeteAnmeldung?: string;
  referenzVorgangId?: string;
}

export interface Marktteilnehmer {
  marktrolle?: string;
  rollencodenummer?: string;
}
```

Entity interfaces are shared across PIDs. If PID 55001 and 55002 both produce `Marktlokation`, their fields are unioned into a single interface.

### Per-PID Response Types

Composes entity interfaces into the actual shape returned by `map_forward`:

```typescript
export interface Pid55001Response {
  marktlokation?: Marktlokation & {
    marktlokationEdifact?: MarktlokationEdifact;
  };
  messlokation?: Messlokation;
  prozessdaten?: Prozessdaten;
  marktteilnehmer?: Marktteilnehmer[];
  ansprechpartner?: Ansprechpartner[];
  nachricht?: Nachricht;
}
```

Companion types are nested under their parent entity via intersection (`&`), matching the engine's `deep_merge_insert` behavior.

## Generation Logic

Three passes over the TOML mappings and PID schema:

### Pass 1: Collect Entity Fields from TOMLs

For each `.toml` file in a PID's mapping directory:
- Read `meta.entity` and `meta.companion_type`.
- Iterate `[fields]`: each mapping with a non-empty `target` becomes a field on the entity interface.
- Iterate `[companion_fields]`: each mapping becomes a field on the companion interface.
- Dotted targets (e.g. `"address.city"`) produce nested interfaces.

### Pass 2: Determine Field Types from PID Schema

Cross-reference each TOML source path (e.g. `cci.2`) against the PID schema JSON:
- `"type": "code"` fields → typed as `CodeField`.
- `"type": "data"` fields → typed as `string`.
- Fields with `enum_map` → string literal union of BO4E-side values (e.g. `"HAUSHALTSKUNDE" | "KEIN_HAUSHALTSKUNDE"`).

### Pass 3: Compose PID Response

- Group entities by their camelCase key.
- If multiple TOMLs target the same entity, union their fields (mirrors `deep_merge_insert`).
- If a `companion_type` exists, nest it under the entity via `&` intersection.
- Determine array vs object from MIG structure: groups that allow repetition (SG2 Marktteilnehmer) become `Entity[]`, groups with discriminators become `Entity`.

### Array vs Object Detection

Inferred from the MIG XML / PID schema group structure:
- Entity with `discriminator` → always single object (`Entity`)
- Entity from a non-repeating source group → single object
- Entity from a repeating group without discriminator → array (`Entity[]`)
- SG2 (Marktteilnehmer) always repeats → array

No explicit TOML hints needed; the structural information is already in the PID schema.

## CLI Interface

New subcommand on `automapper-generator`:

```bash
# Single PID
cargo run -p automapper-generator -- generate-typescript \
  --pid-schema crates/mig-types/src/generated/fv2504/utilmd/pids/pid_55001_schema.json \
  --mappings-dir mappings/FV2504/UTILMD_Strom/pid_55001/ \
  --output-dir generated/typescript/FV2504/UTILMD_Strom/

# Batch mode — all PIDs for a message type
cargo run -p automapper-generator -- generate-typescript \
  --format-version FV2504 \
  --message-type UTILMD \
  --variant Strom \
  --output-dir generated/typescript/FV2504/UTILMD_Strom/
```

**Inputs:**
1. PID schema JSON — code vs data typing, enum values, group repetition
2. TOML mapping directory — entity names, field targets, companion types

**Outputs:**
- `common.d.ts` — shared types (idempotent)
- `pid_NNNNN.d.ts` — per-PID entity interfaces + response type
- `index.d.ts` — barrel re-exports

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Output format | TypeScript `.d.ts` | Primary consumers are frontend TS/JS clients |
| Granularity | Per-entity + per-PID | Entity reuse across PIDs, PID-specific precision |
| Code field handling | `string \| CodeValue` union | Single type covers both `enrich_codes` modes |
| All fields optional | Yes (`?`) | EDIFACT messages may omit any field |
| Array detection | Infer from MIG structure | No TOML changes needed, info already available |
| Generator location | `automapper-generator` crate | Reuses existing PID schema + TOML parsing |
| Generated files | Committed to `generated/typescript/` | Follows project convention for generated code |

## Implementation Scope

### In Scope
- `generate-typescript` subcommand with single-PID and batch modes
- `common.d.ts` with `CodeField` and `CodeValue`
- Per-PID `.d.ts` files with entity interfaces and response types
- `index.d.ts` barrel exports
- Generation for PID 55001 and 55002 as initial targets

### Out of Scope (Future)
- JSON Schema generation (could be added later as alternative output)
- Runtime validation from generated schemas
- Reverse mapping types (BO4E → EDIFACT input shapes)
- Automatic regeneration in CI (manual for now, like other generated code)
