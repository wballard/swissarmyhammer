# Troubleshooting

This guide helps you resolve common issues with SwissArmyHammer. For additional support, check the [GitHub Issues](https://github.com/wballard/swissarmyhammer/issues).

## Quick Diagnostics

Run the doctor command for automated diagnosis:

```bash
swissarmyhammer doctor --verbose
```

## Installation Issues

### Command Not Found

**Problem**: `swissarmyhammer: command not found`

**Solutions**:

1. **Verify installation**:
   ```bash
   ls -la ~/.local/bin/swissarmyhammer
   # or
   ls -la /usr/local/bin/swissarmyhammer
   ```

2. **Add to PATH**:
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

3. **Reinstall**:
   ```bash
   curl -sSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/install.sh | bash
   ```

### Permission Denied

**Problem**: `Permission denied` when running swissarmyhammer

**Solutions**:

```bash
# Make executable
chmod +x $(which swissarmyhammer)

# If installed system-wide, use sudo
sudo chmod +x /usr/local/bin/swissarmyhammer
```

### Installation Script Fails

**Problem**: Install script errors or hangs

**Solutions**:

1. **Manual installation**:
   ```bash
   # Download binary directly
   curl -L https://github.com/wballard/swissarmyhammer/releases/latest/download/swissarmyhammer-linux-x64 -o swissarmyhammer
   chmod +x swissarmyhammer
   sudo mv swissarmyhammer /usr/local/bin/
   ```

2. **Build from source**:
   ```bash
   git clone https://github.com/wballard/swissarmyhammer.git
   cd swissarmyhammer
   cargo build --release
   sudo cp target/release/swissarmyhammer /usr/local/bin/
   ```

## MCP Server Issues

### Server Won't Start

**Problem**: `swissarmyhammer serve` fails to start

**Solutions**:

1. **Check port availability**:
   ```bash
   # Default port
   lsof -i :8080
   
   # Try different port
   swissarmyhammer serve --port 8081
   ```

2. **Debug mode**:
   ```bash
   swissarmyhammer serve --debug
   ```

3. **Check permissions**:
   ```bash
   # Ensure read access to prompt directories
   ls -la ~/.swissarmyhammer/prompts
   ```

### Claude Code Connection Issues

**Problem**: SwissArmyHammer doesn't appear in Claude Code

**Solutions**:

1. **Verify MCP configuration**:
   ```bash
   claude mcp list
   ```

2. **Re-add server**:
   ```bash
   claude mcp remove swissarmyhammer
   claude mcp add swissarmyhammer swissarmyhammer serve
   ```

3. **Check server is running**:
   ```bash
   # In another terminal
   ps aux | grep swissarmyhammer
   ```

4. **Restart Claude Code**:
   - Close Claude Code completely
   - Start Claude Code
   - Check MCP servers are connected

### MCP Protocol Errors

**Problem**: Protocol errors in Claude Code logs

**Solutions**:

1. **Update SwissArmyHammer**:
   ```bash
   # Check version
   swissarmyhammer --version
   
   # Update to latest
   curl -sSL https://install.sh | bash
   ```

2. **Check logs**:
   ```bash
   # Enable debug logging
   swissarmyhammer serve --debug > debug.log 2>&1
   ```

3. **Validate prompt syntax**:
   ```bash
   swissarmyhammer doctor --check prompts
   ```

## Prompt Issues

### Prompts Not Loading

**Problem**: Prompts don't appear or are outdated

**Solutions**:

1. **Check directories**:
   ```bash
   # List prompt directories
   ls -la ~/.swissarmyhammer/prompts
   ls -la ./.swissarmyhammer/prompts
   ```

2. **Validate prompts**:
   ```bash
   swissarmyhammer test <prompt-name>
   swissarmyhammer doctor --check prompts --verbose
   ```

3. **Force reload**:
   ```bash
   # Restart server
   # Ctrl+C to stop, then:
   swissarmyhammer serve
   ```

### Invalid YAML Front Matter

**Problem**: YAML parsing errors

**Common Issues**:

1. **Missing quotes**:
   ```yaml
   # Bad
   description: This won't work: because of the colon
   
   # Good
   description: "This works: because it's quoted"
   ```

2. **Incorrect indentation**:
   ```yaml
   # Bad
   arguments:
   - name: test
   description: Test argument
   
   # Good
   arguments:
     - name: test
       description: Test argument
   ```

3. **Missing required fields**:
   ```yaml
   # Must have name, title, description
   ---
   name: my-prompt
   title: My Prompt
   description: What this prompt does
   ---
   ```

### Template Rendering Errors

**Problem**: Liquid template errors

**Common Issues**:

1. **Undefined variables**:
   ```liquid
   # Error: undefined variable 'foo'
   {{ foo }}
   
   # Fix: Check if variable exists
   {% if foo %}{{ foo }}{% endif %}
   ```

2. **Invalid filter**:
   ```liquid
   # Error: unknown filter
   {{ text | invalid_filter }}
   
   # Fix: Use valid filter
   {{ text | capitalize }}
   ```

3. **Syntax errors**:
   ```liquid
   # Error: unclosed tag
   {% if condition %}
   
   # Fix: Close all tags
   {% if condition %}...{% endif %}
   ```

### Duplicate Prompt Names

**Problem**: Multiple prompts with same name

**Solutions**:

1. **Check override hierarchy**:
   ```bash
   swissarmyhammer list --verbose | grep "prompt-name"
   ```

2. **Rename conflicts**:
   - Local prompts override user prompts
   - User prompts override built-in prompts
   - Rename one to avoid confusion

## Performance Issues

### Slow Prompt Loading

**Problem**: Server takes long to start or reload

**Solutions**:

1. **Disable file watching**:
   ```bash
   swissarmyhammer serve --watch false
   ```

2. **Limit prompt directories**:
   ```bash
   swissarmyhammer serve --prompts ./essential-prompts --builtin false
   ```

3. **Check directory size**:
   ```bash
   find ~/.swissarmyhammer/prompts -type f | wc -l
   ```

### High Memory Usage

**Problem**: Excessive memory consumption

**Solutions**:

1. **Monitor usage**:
   ```bash
   top | grep swissarmyhammer
   ```

2. **Optimize configuration**:
   ```bash
   # Disable file watching
   swissarmyhammer serve --watch false
   
   # Reduce prompt count
   # Move unused prompts to archive
   ```

3. **System limits**:
   ```bash
   # Check ulimits
   ulimit -a
   
   # Increase if needed
   ulimit -n 4096
   ```

## File System Issues

### Permission Errors

**Problem**: Cannot read/write prompt files

**Solutions**:

1. **Fix directory permissions**:
   ```bash
   chmod -R 755 ~/.swissarmyhammer
   chmod -R 644 ~/.swissarmyhammer/prompts/*.md
   ```

2. **Check ownership**:
   ```bash
   ls -la ~/.swissarmyhammer/
   # Fix if needed
   chown -R $USER:$USER ~/.swissarmyhammer
   ```

### File Watching Not Working

**Problem**: Changes to prompts not detected

**Solutions**:

1. **Check file system support**:
   ```bash
   # macOS
   fs_usage | grep swissarmyhammer
   
   # Linux
   inotifywait -m ~/.swissarmyhammer/prompts
   ```

2. **Increase watch limits (Linux)**:
   ```bash
   echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
   sudo sysctl -p
   ```

3. **Manual reload**:
   - Restart the server
   - Or disable watching: `--watch false`

## CLI Command Issues

### Test Command Fails

**Problem**: `swissarmyhammer test` errors

**Solutions**:

1. **Check prompt exists**:
   ```bash
   swissarmyhammer list | grep "prompt-name"
   ```

2. **Validate arguments**:
   ```bash
   # Show required arguments
   swissarmyhammer test prompt-name --help
   ```

3. **Debug mode**:
   ```bash
   swissarmyhammer test prompt-name --debug
   ```

### Export/Import Errors

**Problem**: Cannot export or import prompts

**Solutions**:

1. **Check file permissions**:
   ```bash
   # For export
   touch test-export.tar.gz
   
   # For import
   ls -la import-file.tar.gz
   ```

2. **Validate archive**:
   ```bash
   tar -tzf archive.tar.gz
   ```

3. **Manual export**:
   ```bash
   tar -czf prompts.tar.gz -C ~/.swissarmyhammer prompts/
   ```

## Environment-Specific Issues

### macOS Issues

**Problem**: Security warnings or quarantine

**Solutions**:

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/swissarmyhammer

# Allow in Security & Privacy settings
# System Preferences > Security & Privacy > General
```

### Linux Issues

**Problem**: Library dependencies missing

**Solutions**:

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install libssl-dev

# Fedora
sudo dnf install openssl-devel

# Check dependencies
ldd $(which swissarmyhammer)
```

### Windows Issues

**Problem**: Path or execution issues

**Solutions**:

1. **Use PowerShell as Administrator**
2. **Add to PATH**:
   ```powershell
   $env:Path += ";C:\Program Files\swissarmyhammer"
   [Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::User)
   ```

3. **Windows Defender**:
   - Add exclusion for swissarmyhammer.exe
   - Check Windows Security logs

## Debug Techniques

### Enable Verbose Logging

```bash
# Server debug mode
swissarmyhammer serve --debug

# Redirect to file
swissarmyhammer serve --debug > debug.log 2>&1

# CLI debug
RUST_LOG=debug swissarmyhammer test prompt-name
```

### Check Configuration

```bash
# Run comprehensive checks
swissarmyhammer doctor --verbose

# Check specific areas
swissarmyhammer doctor --check prompts --check mcp

# Auto-fix issues
swissarmyhammer doctor --fix
```

### Trace MCP Communication

```bash
# Save MCP messages
swissarmyhammer serve --debug | grep MCP > mcp-trace.log

# Monitor in real-time
swissarmyhammer serve --debug | grep -E "(request|response)"
```

## Getting Help

### Documentation

- Check this troubleshooting guide first
- Read the [CLI Reference](./cli-reference.md)
- Review [Configuration](./configuration.md) options

### Community Support

- [GitHub Issues](https://github.com/wballard/swissarmyhammer/issues)
- [Discussions](https://github.com/wballard/swissarmyhammer/discussions)
- [Discord/Slack Community](#) (if available)

### Reporting Issues

When reporting issues, include:

1. **System information**:
   ```bash
   swissarmyhammer doctor --json > diagnosis.json
   ```

2. **Steps to reproduce**

3. **Error messages and logs**

4. **Expected vs actual behavior**

### Debug Information Script

Save this as `debug-info.sh`:

```bash
#!/bin/bash
echo "=== SwissArmyHammer Debug Information ==="
echo "Date: $(date)"
echo "Version: $(swissarmyhammer --version)"
echo "OS: $(uname -a)"
echo ""
echo "=== Doctor Report ==="
swissarmyhammer doctor --verbose
echo ""
echo "=== Configuration ==="
cat ~/.swissarmyhammer/config.toml 2>/dev/null || echo "No config file"
echo ""
echo "=== Prompt Directories ==="
ls -la ~/.swissarmyhammer/prompts 2>/dev/null || echo "No user prompts"
ls -la ./.swissarmyhammer/prompts 2>/dev/null || echo "No local prompts"
echo ""
echo "=== Process Check ==="
ps aux | grep swissarmyhammer | grep -v grep
```

Run and save output:
```bash
bash debug-info.sh > debug-info.txt
```

## Common Error Messages

### "Failed to bind to address"
- Port already in use
- Try: `--port 8081`

### "Permission denied"
- File/directory permissions issue
- Try: `chmod +x` or check ownership

### "YAML parse error"
- Invalid YAML syntax in prompt
- Check indentation and special characters

### "Template compilation failed"
- Liquid syntax error
- Check tags are closed and filters exist

### "Prompt not found"
- Prompt name doesn't exist
- Check: `swissarmyhammer list`

### "Connection refused"
- MCP server not running
- Start server: `swissarmyhammer serve`

## Prevention Tips

1. **Regular maintenance**:
   ```bash
   # Weekly health check
   swissarmyhammer doctor
   
   # Update regularly
   swissarmyhammer --version
   ```

2. **Backup prompts**:
   ```bash
   # Regular backups
   swissarmyhammer export ~/.swissarmyhammer/backups/prompts-$(date +%Y%m%d).tar.gz
   ```

3. **Test changes**:
   ```bash
   # Before committing
   swissarmyhammer test new-prompt
   swissarmyhammer doctor --check prompts
   ```

4. **Monitor logs**:
   ```bash
   # Keep logs for debugging
   swissarmyhammer serve --debug > server.log 2>&1 &
   ```