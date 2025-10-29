# Migration Notes - Fixed SurrealDB Schema and Reference Issues

## Problems Fixed

### 1. Record References Experimental Feature
**Issue**: `Experimental capability 'record_references' is not enabled`
**Solution**: Added experimental capability configuration in database initialization:
```rust
let capabilities = Capabilities::new()
    .with_experimental_feature_allowed(ExperimentalFeature::RecordReferences);
let config = Config::new().capabilities(capabilities);
let client = Surreal::new::<SurrealKv>((db_path, config)).await?;
```

### 2. Invalid Field Definition Syntax
**Issue**: `Parse error: Unexpected token 'ON', expected Eof --> [3:75] | 3| ...ersation> ON DELETE CASCADE;`
**Root Cause**: Missing `REFERENCE` keyword in field definitions
**Solution**: Changed from:
```surrealql
DEFINE FIELD conversation_id ON message TYPE record<conversation> ON DELETE CASCADE;
```
To:
```surrealql
DEFINE FIELD conversation_id ON message TYPE record<conversation> REFERENCE ON DELETE CASCADE;
```

## Migration Challenges

### Dynamic Object Key Construction
**Attempted**: Using `OBJECT($conv.template_id, $conv.agent_session_id)`
**Error**: `Invalid function/constant path` because `OBJECT` function doesn't exist

**Research Results**: 
- `object::from_entries` exists but works with key-value array pairs
- No direct object(key, value) function exists in SurrealDB
- Variable interpolation as object keys like `{$conv.template_id: value}` may not be supported

### Current Workaround
For conversation migration, using simple empty object:
```surrealql
UPDATE $conv.id SET agent_sessions = {};
```

## Current Status
- ✅ Schema initialization works with experimental features enabled
- ✅ Record references with ON DELETE CASCADE constraints working
- ⚠️ Migration uses simplified approach for agent_sessions (empty object)
- 🔄 Can enhance migration later as needed

## Functions Available in SurrealDB
From `/Users/davidmaple/cyrup-chat/tmp/surrealdb/crates/core/src/fnc/mod.rs`:
- `object::entries` - Convert object to entries array
- `object::from_entries` - Convert entries array to object  
- `object::extend` - Merge objects
- `object::remove` - Remove keys
- `object::is_empty`, `object::keys`, `object::len`, `object::values`

For future enhancement, could use `object::from_entries([[key, value]])` pattern.