# Implement Memoranda Storage Filesystem Backend

## Overview
Create the filesystem storage backend for memoranda, following the existing patterns from the issues module. This will handle CRUD operations for memos stored in `./.swissarmyhammer/memos/`.

## Tasks

### 1. Create Storage Trait and Implementation
- Create `swissarmyhammer/src/memoranda/storage.rs` following `issues/mod.rs` patterns
- Implement `MemoStorage` trait with async methods:
  - `create_memo(title: String, content: String) -> Result<Memo>`
  - `get_memo(id: &MemoId) -> Result<Memo>`
  - `update_memo(id: &MemoId, content: String) -> Result<Memo>`
  - `delete_memo(id: &MemoId) -> Result<()>`
  - `list_memos() -> Result<Vec<Memo>>`
  - `search_memos(query: &str) -> Result<Vec<Memo>>`

### 2. Directory Management
- Ensure `.swissarmyhammer/memos/` directory is created on first use
- Follow patterns from existing directory utilities
- Proper error handling for permission issues

### 3. File Storage Format
- Store each memo as individual JSON file: `{memo_id}.json`
- Use serde for serialization/deserialization
- Handle file corruption and recovery

### 4. Error Integration
- Add `MemoStorageError` variants to main error enum:
  - `MemoNotFound(MemoId)`
  - `MemoStorageIo(std::io::Error)`
  - `MemoSerialization(serde_json::Error)`

## Implementation Notes
- Follow existing `IssueStorage` patterns exactly
- Use ULID-based filenames for natural sorting
- Async/await throughout for consistency
- Comprehensive error handling

## Acceptance Criteria
- [ ] MemoStorage trait implemented with all CRUD operations
- [ ] Directory creation working with proper permissions
- [ ] JSON file storage working reliably
- [ ] Error types integrated with main error system
- [ ] Basic search functionality (exact string matching)
- [ ] Unit tests covering all storage operations