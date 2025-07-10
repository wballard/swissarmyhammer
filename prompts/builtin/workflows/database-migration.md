---
name: database-migration
title: Database Migration
description: Robust database migration workflow with comprehensive error handling and recovery
category: workflows
tags:
  - database
  - migration
  - error-handling
  - example
arguments:
  - name: migration_path
    description: Path to the directory containing migration files
    required: false
    default: "migrations/"
    type_hint: string
  - name: target_version
    description: Target database version to migrate to
    required: false
    default: "latest"
    type_hint: string
  - name: backup_enabled
    description: Whether to create a backup before migration
    required: false
    default: "true"
    type_hint: string
  - name: dry_run
    description: Whether to perform a dry run without making changes
    required: false
    default: "false"
    type_hint: string
---

# Database Migration Workflow

This workflow demonstrates comprehensive error handling with backup, recovery,
retry logic, and graceful degradation for database migrations.

```mermaid
stateDiagram-v2
    [*] --> PreCheck: Start Migration
    PreCheck --> CreateBackup: Checks passed
    PreCheck --> PreCheckFailed: Checks failed
    CreateBackup --> ValidateBackup: Backup created
    CreateBackup --> BackupFailed: Backup failed
    ValidateBackup --> ApplyMigrations: Backup valid
    ValidateBackup --> BackupInvalid: Backup invalid
    ApplyMigrations --> VerifyMigration: Success
    ApplyMigrations --> MigrationError: Failed
    VerifyMigration --> PostCheck: Verified
    VerifyMigration --> VerificationFailed: Verification failed
    PostCheck --> Success: All checks passed
    PostCheck --> PostCheckFailed: Checks failed
    MigrationError --> RetryDecision: Error caught
    RetryDecision --> ApplyMigrations: Retry
    RetryDecision --> Rollback: No retry
    VerificationFailed --> Rollback: Initiate rollback
    PostCheckFailed --> Rollback: Initiate rollback
    Rollback --> RestoreBackup: Migrations rolled back
    Rollback --> RollbackFailed: Rollback failed
    RestoreBackup --> ValidateRestore: Backup restored
    RestoreBackup --> RestoreFailed: Restore failed
    ValidateRestore --> RecoveryComplete: Restore valid
    ValidateRestore --> CriticalError: Restore invalid
    BackupFailed --> AbortMigration: Cannot proceed
    BackupInvalid --> AbortMigration: Cannot proceed
    PreCheckFailed --> [*]: Pre-checks failed
    AbortMigration --> [*]: Migration aborted
    Success --> [*]: Migration successful
    RecoveryComplete --> [*]: Recovered to previous state
    RollbackFailed --> EmergencyMode: Enter emergency mode
    RestoreFailed --> EmergencyMode: Enter emergency mode
    CriticalError --> EmergencyMode: Enter emergency mode
    EmergencyMode --> [*]: Manual intervention required
    
    PreCheck: Pre-Migration Checks
    PreCheck: action: execute_prompt
    PreCheck: prompt: database/pre-migration-check
    PreCheck: variables:
    PreCheck:   target_version: "{{ target_version }}"
    PreCheck:   current_version: "{{ current_db_version }}"
    PreCheck: error_handler: log_and_continue
    
    CreateBackup: Create Database Backup
    CreateBackup: action: execute_prompt
    CreateBackup: prompt: database/create-backup
    CreateBackup: variables:
    CreateBackup:   backup_name: "pre_migration_{{ timestamp }}"
    CreateBackup:   compression: "true"
    CreateBackup: timeout: 1800
    CreateBackup: retry:
    CreateBackup:   attempts: 3
    CreateBackup:   delay: 60
    
    ValidateBackup: Validate Backup
    ValidateBackup: action: execute_prompt
    ValidateBackup: prompt: database/validate-backup
    ValidateBackup: variables:
    ValidateBackup:   backup_id: "{{ CreateBackup.backup_id }}"
    ValidateBackup:   validation_level: "comprehensive"
    
    ApplyMigrations: Apply Database Migrations
    ApplyMigrations: action: execute_prompt
    ApplyMigrations: prompt: database/apply-migrations
    ApplyMigrations: variables:
    ApplyMigrations:   migration_path: "{{ migration_path }}"
    ApplyMigrations:   target: "{{ target_version }}"
    ApplyMigrations:   dry_run: "{{ dry_run }}"
    ApplyMigrations:   transaction_mode: "true"
    ApplyMigrations: error_capture: true
    ApplyMigrations: timeout: 3600
    
    VerifyMigration: Verify Migration Success
    VerifyMigration: action: execute_prompt
    VerifyMigration: prompt: database/verify-migration
    VerifyMigration: variables:
    VerifyMigration:   expected_version: "{{ target_version }}"
    VerifyMigration:   check_integrity: "true"
    
    PostCheck: Post-Migration Validation
    PostCheck: action: parallel_execute
    PostCheck: error_mode: collect_all
    PostCheck: tasks:
    PostCheck:   - action: execute_prompt
    PostCheck:     prompt: database/check-schema
    PostCheck:   - action: execute_prompt
    PostCheck:     prompt: database/check-data-integrity
    PostCheck:   - action: execute_prompt
    PostCheck:     prompt: database/performance-baseline
    
    MigrationError: Handle Migration Error
    MigrationError: action: execute_prompt
    MigrationError: prompt: database/analyze-error
    MigrationError: variables:
    MigrationError:   error: "{{ ApplyMigrations.error }}"
    MigrationError:   context: "{{ ApplyMigrations.context }}"
    
    RetryDecision: Decide on Retry
    RetryDecision: action: conditional
    RetryDecision: condition: "{{ MigrationError.retryable == 'true' and retry_count < 3 }}"
    RetryDecision: on_true:
    RetryDecision:   action: set_variable
    RetryDecision:   variable: retry_count
    RetryDecision:   value: "{{ retry_count + 1 }}"
    RetryDecision: on_false:
    RetryDecision:   action: log
    RetryDecision:   message: "Migration failed after {{ retry_count }} attempts"
    
    Rollback: Rollback Migrations
    Rollback: action: execute_prompt
    Rollback: prompt: database/rollback-migrations
    Rollback: variables:
    Rollback:   target_version: "{{ PreCheck.initial_version }}"
    Rollback:   force: "true"
    Rollback: error_handler: continue_and_log
    
    RestoreBackup: Restore from Backup
    RestoreBackup: action: execute_prompt
    RestoreBackup: prompt: database/restore-backup
    RestoreBackup: variables:
    RestoreBackup:   backup_id: "{{ CreateBackup.backup_id }}"
    RestoreBackup:   validate_before: "true"
    RestoreBackup: timeout: 3600
    
    ValidateRestore: Validate Restored Database
    ValidateRestore: action: execute_prompt
    ValidateRestore: prompt: database/validate-restore
    ValidateRestore: variables:
    ValidateRestore:   expected_version: "{{ PreCheck.initial_version }}"
    ValidateRestore:   full_validation: "true"
    
    EmergencyMode: Emergency Recovery Mode
    EmergencyMode: action: parallel_execute
    EmergencyMode: tasks:
    EmergencyMode:   - action: execute_prompt
    EmergencyMode:     prompt: notifications/alert-critical
    EmergencyMode:     variables:
    EmergencyMode:       severity: "critical"
    EmergencyMode:       message: "Database migration failed critically"
    EmergencyMode:   - action: execute_prompt
    EmergencyMode:     prompt: database/generate-recovery-plan
    EmergencyMode:     variables:
    EmergencyMode:       failure_context: "{{ all_errors }}"
    EmergencyMode:   - action: set_variable
    EmergencyMode:     variable: database_mode
    EmergencyMode:     value: "read_only"
    
    PreCheckFailed: Pre-Check Failed
    PreCheckFailed: action: log
    PreCheckFailed: level: error
    PreCheckFailed: message: "Pre-migration checks failed: {{ PreCheck.failures }}"
    
    BackupFailed: Backup Failed
    BackupFailed: action: log
    BackupFailed: level: critical
    BackupFailed: message: "Failed to create backup after {{ CreateBackup.retry_count }} attempts"
    
    BackupInvalid: Invalid Backup
    BackupInvalid: action: log
    BackupInvalid: level: critical
    BackupInvalid: message: "Backup validation failed: {{ ValidateBackup.errors }}"
    
    AbortMigration: Abort Migration
    AbortMigration: action: set_variable
    AbortMigration: variable: migration_status
    AbortMigration: value: "aborted"
    AbortMigration: output: "Migration aborted due to backup issues. No changes made."
    
    Success: Migration Complete
    Success: action: parallel_execute
    Success: tasks:
    Success:   - action: set_variable
    Success:     variable: migration_status
    Success:     value: "completed"
    Success:   - action: execute_prompt
    Success:     prompt: database/cleanup-old-backups
    Success:     variables:
    Success:       keep_count: "3"
    Success: output: "Migration completed successfully to version {{ target_version }}"
    
    RecoveryComplete: Recovery Successful
    RecoveryComplete: action: set_variable
    RecoveryComplete: variable: migration_status
    RecoveryComplete: value: "rolled_back"
    RecoveryComplete: output: "Successfully recovered to previous state"
    
    VerificationFailed: Verification Failed
    VerificationFailed: action: log
    VerificationFailed: level: error
    VerificationFailed: message: "Migration verification failed: {{ VerifyMigration.issues }}"
    
    PostCheckFailed: Post-Check Failed
    PostCheckFailed: action: log
    PostCheckFailed: level: error
    PostCheckFailed: message: "Post-migration validation failed: {{ PostCheck.failures }}"
    
    RollbackFailed: Rollback Failed
    RollbackFailed: action: log
    RollbackFailed: level: critical
    RollbackFailed: message: "Failed to rollback migrations: {{ Rollback.error }}"
    
    RestoreFailed: Restore Failed
    RestoreFailed: action: log
    RestoreFailed: level: critical
    RestoreFailed: message: "Failed to restore from backup: {{ RestoreBackup.error }}"
    
    CriticalError: Critical Validation Error
    CriticalError: action: log
    CriticalError: level: critical
    CriticalError: message: "Database restore validation failed: {{ ValidateRestore.errors }}"
```

## Usage

Run this workflow with:

```bash
# Standard migration with all safety features
swissarmyhammer workflow run database-migration --set target_version=v2.0

# Dry run to test migrations
swissarmyhammer workflow run database-migration \
  --set target_version=v2.0 \
  --set dry_run=true

# Migration without backup (dangerous!)
swissarmyhammer workflow run database-migration \
  --set target_version=v2.0 \
  --set backup_enabled=false
```

## Error Handling Features

1. **Multiple Error States**: Different handlers for different failure types
2. **Retry Logic**: Automatic retry for transient failures
3. **Rollback Mechanisms**: Multiple levels of rollback (migration rollback, backup restore)
4. **Emergency Mode**: Critical failure handling with notifications
5. **Error Propagation**: Errors captured and passed to decision states
6. **Graceful Degradation**: System switches to read-only mode on critical failure
7. **Validation at Every Step**: Ensures system integrity throughout