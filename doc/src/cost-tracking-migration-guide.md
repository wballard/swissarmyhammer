# Cost Tracking Migration Guide

This guide helps you add cost tracking to existing SwissArmyHammer projects, upgrade configurations, and migrate data between different setups.

## Table of Contents

- [Adding Cost Tracking to Existing Projects](#adding-cost-tracking-to-existing-projects)
- [Configuration Migration](#configuration-migration)
- [Data Migration](#data-migration)
- [Version Migration](#version-migration)
- [Deployment Migration](#deployment-migration)

## Adding Cost Tracking to Existing Projects

### Prerequisites

Before enabling cost tracking in an existing project:

1. **Backup Configuration**: Save your current `swissarmyhammer.yaml`
2. **Update SwissArmyHammer**: Ensure you're running a version that supports cost tracking
3. **Check Permissions**: Ensure write permissions for database files (if using database)
4. **Review Claude Plan**: Know your current Claude pricing tier

### Step-by-Step Migration

#### Step 1: Verify Current Setup

Check your current SwissArmyHammer installation:

```bash
# Check version (cost tracking available in v1.5.0+)
swissarmyhammer --version

# Verify current configuration
swissarmyhammer doctor

# Backup existing configuration
cp swissarmyhammer.yaml swissarmyhammer.yaml.backup
```

#### Step 2: Add Minimal Cost Tracking

Start with basic cost tracking configuration:

```yaml
# Add this to your existing swissarmyhammer.yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"  # or "max" for unlimited plans
  rates:
    input_token_cost: 0.000015   # Adjust for your Claude plan
    output_token_cost: 0.000075
```

#### Step 3: Test Basic Functionality

Verify cost tracking works without affecting existing workflows:

```bash
# Test configuration
swissarmyhammer doctor

# Should show: "✓ Cost tracking: enabled"

# Process a simple test issue
echo "# Test Issue\n\nImplement a simple hello world function." > test-cost-tracking.md
swissarmyhammer issue work test-cost-tracking.md

# Check if cost report was added to the completed issue
cat test-cost-tracking.md
```

#### Step 4: Gradually Add Features

Once basic tracking works, add advanced features incrementally:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Add detailed reporting
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    cost_precision_decimals: 4
  
  # Add session management (optional)
  session_management:
    max_concurrent_sessions: 100
    session_timeout_hours: 24
```

#### Step 5: Enable Database Storage (Optional)

For persistent storage and analytics:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  reporting:
    include_in_issues: true
    detailed_breakdown: true
  
  # Add database storage
  database:
    enabled: true
    file_path: "./costs.db"
    retention_days: 90
  
  # Enable aggregation
  aggregation:
    enabled: true
    retention_days: 90
```

### Migration Checklist

- [ ] Backup existing configuration
- [ ] Update SwissArmyHammer to cost tracking-compatible version
- [ ] Add minimal cost tracking configuration
- [ ] Test with simple issue to verify functionality
- [ ] Check that existing workflows still work normally
- [ ] Gradually add advanced features (reporting, database, aggregation)
- [ ] Update CI/CD pipelines if needed
- [ ] Train team on new cost tracking features
- [ ] Monitor performance impact

## Configuration Migration

### From Environment Variables to YAML

If you previously used environment variables for configuration, migrate them to YAML:

**Old Environment Variables**:
```bash
export SAH_API_KEY="your-api-key"
export SAH_MODEL="claude-3-sonnet"
export SAH_MAX_TOKENS="4096"
```

**New YAML Configuration**:
```yaml
# Keep existing configuration
api_key: "your-api-key"
model: "claude-3-sonnet" 
max_tokens: 4096

# Add cost tracking
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

### From Basic to Advanced Configuration

**Migration Path: Basic → Advanced**

**Current Basic Config**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

**Migrated Advanced Config**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Add advanced session management
  session_management:
    max_concurrent_sessions: 200     # Scale based on usage
    session_timeout_hours: 48        # Longer retention
    cleanup_interval_hours: 4        # More frequent cleanup
    max_api_calls_per_session: 1000  # Higher limits
  
  # Add database for persistence
  database:
    enabled: true
    file_path: "/data/costs.db"      # Persistent location
    retention_days: 365              # Full year retention
    connection_timeout_seconds: 60
    max_connections: 20
  
  # Add comprehensive reporting
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    currency_locale: "en-US"
    include_performance_metrics: true
    cost_precision_decimals: 6
  
  # Add aggregation analytics
  aggregation:
    enabled: true
    retention_days: 180
    max_stored_sessions: 50000
```

### Multi-Environment Configuration

**Migration Strategy: Single Config → Multi-Environment**

**Create Environment-Specific Configurations**:

`config/base.yaml` (shared configuration):
```yaml
# Shared base configuration
api_key: "${CLAUDE_API_KEY}"
model: "claude-3-sonnet"

cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

`config/development.yaml`:
```yaml
extends: base.yaml

cost_tracking:
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    cost_precision_decimals: 8
  aggregation:
    enabled: true
  database:
    enabled: true
    file_path: "./dev-costs.db"
    retention_days: 7
```

`config/production.yaml`:
```yaml
extends: base.yaml

cost_tracking:
  session_management:
    max_concurrent_sessions: 500
    session_timeout_hours: 12
    cleanup_interval_hours: 2
  database:
    enabled: true
    file_path: "/data/production-costs.db"
    retention_days: 365
    max_connections: 50
  reporting:
    include_in_issues: false  # Disabled for performance
    detailed_breakdown: false
  aggregation:
    enabled: false  # Disabled for performance
```

## Data Migration

### In-Memory to Database Migration

**Scenario**: You have in-memory cost tracking data and want to enable database storage.

**Migration Script** (`migrate-to-database.py`):
```python
#!/usr/bin/env python3
import json
import sqlite3
import os
from datetime import datetime

def migrate_memory_to_database():
    """
    Migrate cost tracking data from memory dumps to SQLite database
    """
    
    # Create database and tables
    conn = sqlite3.connect('costs.db')
    
    # Create tables (matches SwissArmyHammer schema)
    conn.executescript('''
        CREATE TABLE IF NOT EXISTS cost_sessions (
            session_id TEXT PRIMARY KEY,
            issue_id TEXT NOT NULL,
            started_at DATETIME NOT NULL,
            completed_at DATETIME,
            status TEXT NOT NULL,
            total_input_tokens INTEGER DEFAULT 0,
            total_output_tokens INTEGER DEFAULT 0,
            total_cost DECIMAL(10,6),
            metadata TEXT
        );
        
        CREATE TABLE IF NOT EXISTS api_calls (
            call_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            started_at DATETIME NOT NULL,
            completed_at DATETIME,
            endpoint TEXT NOT NULL,
            model TEXT NOT NULL,
            input_tokens INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            cost DECIMAL(8,6),
            status TEXT NOT NULL,
            error_message TEXT,
            FOREIGN KEY (session_id) REFERENCES cost_sessions(session_id)
        );
    ''')
    
    # Load memory dumps (if available)
    if os.path.exists('cost_tracking_dump.json'):
        with open('cost_tracking_dump.json', 'r') as f:
            data = json.load(f)
            
        # Insert sessions
        for session in data.get('sessions', []):
            conn.execute('''
                INSERT INTO cost_sessions 
                (session_id, issue_id, started_at, completed_at, status,
                 total_input_tokens, total_output_tokens, total_cost)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                session['id'],
                session['issue_id'],
                session['started_at'],
                session.get('completed_at'),
                session['status'],
                session['total_input_tokens'],
                session['total_output_tokens'],
                session.get('total_cost')
            ))
            
            # Insert API calls for this session
            for call in session.get('api_calls', []):
                conn.execute('''
                    INSERT INTO api_calls
                    (call_id, session_id, started_at, completed_at,
                     endpoint, model, input_tokens, output_tokens, status)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    call['id'],
                    session['id'],
                    call['started_at'],
                    call.get('completed_at'),
                    call['endpoint'],
                    call['model'],
                    call['input_tokens'],
                    call['output_tokens'],
                    call['status']
                ))
    
    conn.commit()
    conn.close()
    
    print("Migration completed successfully!")
    print(f"Database created at: {os.path.abspath('costs.db')}")

if __name__ == "__main__":
    migrate_memory_to_database()
```

### Database Schema Migration

**Scenario**: Upgrading SwissArmyHammer changes the database schema.

**Schema Version Check**:
```sql
-- Check current schema version
SELECT name FROM sqlite_master WHERE type='table' AND name='schema_migrations';

-- If table exists, check version
SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;
```

**Migration Script** (`migrate-schema.py`):
```python
#!/usr/bin/env python3
import sqlite3
import sys

def get_schema_version(conn):
    """Get current database schema version"""
    try:
        cursor = conn.execute("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
        row = cursor.fetchone()
        return row[0] if row else 0
    except sqlite3.OperationalError:
        return 0

def migrate_v1_to_v2(conn):
    """Migrate from version 1 to version 2"""
    print("Migrating schema from v1 to v2...")
    
    # Add new columns
    conn.executescript('''
        ALTER TABLE cost_sessions ADD COLUMN metadata TEXT;
        ALTER TABLE api_calls ADD COLUMN cost DECIMAL(8,6);
        
        -- Create new aggregation table
        CREATE TABLE cost_analytics (
            analysis_date DATE PRIMARY KEY,
            total_sessions INTEGER,
            total_api_calls INTEGER,
            total_input_tokens INTEGER,
            total_output_tokens INTEGER,
            total_cost DECIMAL(12,6),
            metrics_json TEXT
        );
        
        -- Update schema version
        INSERT OR REPLACE INTO schema_migrations (version, migrated_at) 
        VALUES (2, datetime('now'));
    ''')

def migrate_database(db_path):
    """Migrate database to latest schema"""
    conn = sqlite3.connect(db_path)
    current_version = get_schema_version(conn)
    target_version = 2  # Current latest version
    
    print(f"Current schema version: {current_version}")
    print(f"Target schema version: {target_version}")
    
    if current_version < target_version:
        # Create schema_migrations table if it doesn't exist
        conn.execute('''
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                migrated_at DATETIME NOT NULL
            )
        ''')
        
        # Run migrations sequentially
        if current_version < 2:
            migrate_v1_to_v2(conn)
        
        print("Database migration completed!")
    else:
        print("Database is already at the latest version.")
    
    conn.close()

if __name__ == "__main__":
    db_path = sys.argv[1] if len(sys.argv) > 1 else "costs.db"
    migrate_database(db_path)
```

## Version Migration

### SwissArmyHammer Version Compatibility

**Version Compatibility Matrix**:

| SwissArmyHammer Version | Cost Tracking Support | Database Version | Migration Required |
|-------------------------|------------------------|------------------|-------------------|
| < 1.5.0 | None | N/A | Add configuration |
| 1.5.0 - 1.5.5 | Basic | 1 | None |
| 1.6.0 - 1.6.5 | Advanced | 2 | Schema migration |
| 1.7.0+ | Full | 3 | Schema + data migration |

### Version Migration Steps

**From Pre-Cost-Tracking Versions (< 1.5.0)**:

1. **Update SwissArmyHammer**:
   ```bash
   # Update to latest version
   curl -sSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/install.sh | sh
   
   # Verify version
   swissarmyhammer --version
   ```

2. **Add Cost Tracking Configuration**:
   ```yaml
   # Add to existing swissarmyhammer.yaml
   cost_tracking:
     enabled: true
     pricing_model: "paid"
     rates:
       input_token_cost: 0.000015
       output_token_cost: 0.000075
   ```

3. **Test Compatibility**:
   ```bash
   swissarmyhammer doctor
   # Should not report any breaking changes
   ```

**From Basic Cost Tracking (1.5.x) to Advanced (1.6.x)**:

1. **Backup Data**:
   ```bash
   # Backup configuration and data
   cp swissarmyhammer.yaml swissarmyhammer.yaml.v1.5
   cp costs.db costs.db.v1.5 2>/dev/null || true
   ```

2. **Update Configuration** (new options available):
   ```yaml
   cost_tracking:
     enabled: true
     pricing_model: "paid"
     rates:
       input_token_cost: 0.000015
       output_token_cost: 0.000075
     
     # New advanced options in 1.6.x
     session_management:
       max_concurrent_sessions: 100
       session_timeout_hours: 24
       cleanup_interval_hours: 6
     
     aggregation:
       enabled: true
       retention_days: 90
   ```

3. **Run Schema Migration**:
   ```bash
   # SwissArmyHammer will automatically migrate schema on first run
   swissarmyhammer doctor
   
   # Or run manual migration
   python migrate-schema.py costs.db
   ```

## Deployment Migration

### Single Instance to Multi-Instance

**Scenario**: Migrating from single SwissArmyHammer instance to load-balanced setup.

**Migration Steps**:

1. **Set up Shared Storage**:
   ```bash
   # Create shared NFS mount
   sudo mkdir -p /shared/swissarmyhammer
   sudo mount nfs-server:/export/swissarmyhammer /shared/swissarmyhammer
   ```

2. **Migrate Database to Shared Location**:
   ```bash
   # Copy existing database to shared location
   cp costs.db /shared/swissarmyhammer/costs.db
   
   # Verify permissions
   chmod 664 /shared/swissarmyhammer/costs.db
   ```

3. **Update Configuration for All Instances**:
   ```yaml
   cost_tracking:
     enabled: true
     pricing_model: "paid"
     rates:
       input_token_cost: 0.000015
       output_token_cost: 0.000075
     
     database:
       enabled: true
       file_path: "/shared/swissarmyhammer/costs.db"  # Shared path
       connection_timeout_seconds: 180  # Higher for network
       max_connections: 200  # Support multiple instances
     
     session_management:
       max_concurrent_sessions: 100  # Per instance
       session_timeout_hours: 24
   ```

### Docker Migration

**From Local to Containerized Deployment**:

**Step 1: Create Docker Configuration**

`docker/swissarmyhammer.yaml`:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  database:
    enabled: true
    file_path: "/data/costs.db"  # Container path
  reporting:
    include_in_issues: true
```

**Step 2: Migrate Data to Container Volume**

```bash
# Create Docker volume
docker volume create swissarmyhammer-data

# Copy existing database to volume
docker run --rm -v swissarmyhammer-data:/data -v $(pwd):/backup alpine \
  cp /backup/costs.db /data/costs.db
```

**Step 3: Deploy Container**

```yaml
# docker-compose.yml
version: '3.8'
services:
  swissarmyhammer:
    image: swissarmyhammer:latest
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
    volumes:
      - swissarmyhammer-data:/data
      - ./docker/swissarmyhammer.yaml:/etc/swissarmyhammer/swissarmyhammer.yaml
    ports:
      - "8080:8080"

volumes:
  swissarmyhammer-data:
```

## Migration Troubleshooting

### Common Migration Issues

**Issue: "Cost tracking disabled after migration"**

**Solution**: Check configuration file path and environment variables:
```bash
# Check which config file is being used
swissarmyhammer doctor --verbose

# Verify environment variables
env | grep SAH_COST

# Test configuration explicitly
swissarmyhammer --config ./swissarmyhammer.yaml doctor
```

**Issue: "Database migration failed"**

**Solution**: Backup and recreate database:
```bash
# Backup existing database
cp costs.db costs.db.backup

# Check database integrity
sqlite3 costs.db "PRAGMA integrity_check;"

# If corrupted, recreate
rm costs.db
swissarmyhammer doctor  # Will create new database
```

**Issue: "Configuration validation errors after migration"**

**Solution**: Use configuration migration tool:
```python
#!/usr/bin/env python3
import yaml

def validate_config(config_file):
    """Validate migrated configuration"""
    with open(config_file, 'r') as f:
        config = yaml.safe_load(f)
    
    if 'cost_tracking' not in config:
        print("❌ cost_tracking section missing")
        return False
    
    ct = config['cost_tracking']
    
    if not ct.get('enabled'):
        print("❌ cost_tracking.enabled is false or missing")
        return False
    
    if ct.get('pricing_model') not in ['paid', 'max']:
        print("❌ Invalid pricing_model")
        return False
    
    if ct.get('pricing_model') == 'paid' and 'rates' not in ct:
        print("❌ rates required for paid pricing model")
        return False
    
    print("✅ Configuration is valid")
    return True

if __name__ == "__main__":
    validate_config('swissarmyhammer.yaml')
```

This migration guide ensures smooth transitions when adding cost tracking to existing projects and upgrading between different configurations and versions.