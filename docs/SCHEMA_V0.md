# FormProof Schema v0 Specification

This document defines the exact JSON Schema subset supported by FormProof v0.
The schema is **frozen** — any schema that passes validation will produce
identical circuits across FormProof versions.

## Grammar

```
Schema      ::= { "type": "object", "properties": Properties, "required": Required }
Properties  ::= { PropertyName: PropertyDef, ... }   // 1 to 8 properties
PropertyDef ::= IntegerDef | EnumDef | StringDef | Bytes32Def
Required    ::= [ PropertyName, ... ]                // subset of property names

IntegerDef  ::= { "type": "integer", "minimum"?: uint64, "maximum"?: uint64 }
EnumDef     ::= { "enum": [ string, ... ] }          // 1 to 8 string variants
StringDef   ::= { "type": "string", "maxLength"?: uint64 }  // maxLength ≤ 64
Bytes32Def  ::= { "type": "string", "format": "bytes32" }
```

## Limits

| Constraint | Limit | Error if Exceeded |
|------------|-------|-------------------|
| Properties per object | ≤ 8 | `TooManyProperties` |
| Enum variants | ≤ 8 per enum | `TooManyEnumVariants` |
| String maxLength | ≤ 64 | `StringTooLong` |
| Bytes32 length | exactly 32 bytes | `InvalidBytes32Length` |

## Property Types

### Integer (`type: "integer"`)

64-bit unsigned integers with optional range constraints.

```json
{
  "amount": {
    "type": "integer",
    "minimum": 0,
    "maximum": 50
  }
}
```

**Constraints:**
- `minimum` (optional): Value must be ≥ this (u64)
- `maximum` (optional): Value must be ≤ this (u64)
- If both specified, `minimum` must be ≤ `maximum`

**Reject rules:**
- Value < minimum → constraint not satisfied
- Value > maximum → constraint not satisfied
- minimum > maximum in schema → `MinGreaterThanMax`

### Enum (`enum: [...]`)

String enumeration with a fixed set of valid values.

```json
{
  "currency": {
    "enum": ["USD", "EUR", "GBP"]
  }
}
```

**Constraints:**
- 1 to 8 string variants
- All variants must be strings
- Witness value must exactly match one variant

**Reject rules:**
- Value not in enum set → constraint not satisfied
- More than 8 variants → `TooManyEnumVariants`
- Non-string variants → `EnumNotStrings`

### String (`type: "string"`)

UTF-8 string with bounded length.

```json
{
  "name": {
    "type": "string",
    "maxLength": 32
  }
}
```

**Constraints:**
- `maxLength` (optional, default 64): Maximum byte length
- Strings longer than maxLength are truncated in the witness

**Reject rules:**
- maxLength > 64 in schema → `StringTooLong`

### Bytes32 (`type: "string", `format: "bytes32"`)

Fixed 32-byte binary data, typically for hashes or identifiers.

```json
{
  "token_id": {
    "type": "string",
    "format": "bytes32"
  }
}
```

**Constraints:**
- Exactly 32 bytes
- In JSON witness, represented as 64-character hex string

**Reject rules:**
- Length ≠ 32 bytes → `InvalidBytes32Length`

## Required Fields

The `required` array lists property names that must be present in the witness.

```json
{
  "type": "object",
  "properties": {
    "amount": { "type": "integer" },
    "currency": { "enum": ["USD", "EUR"] }
  },
  "required": ["amount", "currency"]
}
```

**Reject rules:**
- Required property missing from witness → constraint not satisfied
- Required property name not in properties → `RequiredNotDefined`

## Schema Validation Errors

| Error | Cause |
|-------|-------|
| `NotAnObject` | Schema type is not "object" or missing properties |
| `TooManyProperties` | More than 8 properties defined |
| `TooManyEnumVariants` | Enum has more than 8 variants |
| `StringTooLong` | maxLength exceeds 64 |
| `UnsupportedType` | Property type is not integer, string, or enum |
| `RequiredNotDefined` | Required field not in properties |
| `MissingType` | Property lacks type field (and no enum) |
| `EnumNotStrings` | Enum contains non-string values |
| `MinGreaterThanMax` | minimum > maximum for integer |
| `InvalidBytes32Length` | bytes32 value is not 32 bytes |

## Witness Validation

The circuit enforces these rules at proof time:

1. **Required fields**: All fields in `required` must be present
2. **Integer range**: If present, value must satisfy minimum/maximum
3. **Enum membership**: If present, value must be in the enum set
4. **String length**: If present, length must be ≤ maxLength

A valid proof guarantees all constraints are satisfied. The verifier
learns nothing about actual values — only that they satisfy the schema.

## Commitment

The witness is serialized and hashed (SHA-256) to produce a 32-byte
commitment. This commitment is the only public input to the circuit.

Serialization order:
1. Properties sorted alphabetically by name
2. For each property: presence byte (1 if present, 0 if absent) + value bytes

## Not Supported (v0)

The following JSON Schema features are **not** supported:

- Nested objects
- Arrays
- `$ref` or `$defs`
- `additionalProperties`
- `oneOf`, `anyOf`, `allOf`
- `pattern` (regex)
- `format` (except `bytes32`)
- `const`
- `default`
- Numeric types other than integer (no `number`)
- Signed integers (all values are u64)

These may be added in future versions.
