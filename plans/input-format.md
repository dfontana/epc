# Input Format Feature

## Overview

The `-i`/`--input-format` CLI argument allows users to specify custom datetime string formats using strftime-style specifiers. When not specified, the tool auto-detects timestamps and RFC3339/ISO8601 datetime strings, preserving backward compatibility.

## Implementation Rationale

### Design Decisions

**Parsing Strategy**: The implementation uses a cascading approach that tries formats in this order:
1. Timestamp parsing (always first to preserve existing behavior)
2. Timezone-aware datetime parsing with custom format
3. Timezone-naive datetime parsing (assumes UTC)
4. Date-only parsing (assumes midnight UTC)

This strategy maximizes compatibility while providing clear fallback behavior for incomplete datetime specifications.

**API Changes**: Changed `input: Vec<ConversionInput>` to `input: Vec<String>` to enable format-aware parsing. The new `ConversionInput::from_str_with_format()` method handles the parsing logic, allowing the same input vector to contain both timestamps and formatted datetime strings.

**Error Handling**: All parsing uses Result types with descriptive error messages. No panics occur - invalid formats or unparseable strings return clear error descriptions.

## Supported Formats

### Core Specifiers
- **%Y**: 4-digit year (2023)
- **%m**: Month (01-12) 
- **%d**: Day (01-31)
- **%H**: Hour 24h (00-23)
- **%M**: Minute (00-59)
- **%S**: Second (00-60)
- **%.3f**: Milliseconds (.123)
- **%z**: Timezone offset (+0200)
- **%:z**: Timezone with colon (+02:00)

### Mixed Input Support
Timestamps and custom-formatted strings can be used together in the same command:
```bash
epc convert -i "%Y-%m-%d" 1679258022 "2023-07-15"
```

### Timezone Handling
- Timezone-aware formats preserve the specified offset
- Timezone-naive formats assume UTC
- Date-only formats assume midnight UTC

## Testing Strategy

The implementation includes comprehensive unit tests that validate exact datetime parsing rather than just success/failure. Tests use `Option<DateTime<FixedOffset>>` where `Some(datetime)` specifies the expected parsed result and `None` indicates expected parsing failure.

This approach provides stronger guarantees about parsing correctness and catches subtle bugs in datetime conversion logic.

## Usage Examples

```bash
# European date format
epc convert -i "%d/%m/%Y" "15/07/2023"

# US format with time  
epc convert -i "%m/%d/%Y %H:%M:%S" "07/15/2023 14:30:45"

# ISO format with timezone
epc convert -i "%Y-%m-%d %H:%M:%S%:z" "2023-07-15 14:30:45+02:00"

# Compact format
epc convert -i "%Y%m%d_%H%M%S" "20230715_143045"
```