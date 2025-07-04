# completion

The `completion` command generates shell completion scripts for SwissArmyHammer, enabling tab completion for commands, options, and arguments in your shell.

## Usage

```bash
swissarmyhammer completion <SHELL>
```

### Arguments

- `<SHELL>` - The shell to generate completions for (`bash`, `zsh`, `fish`, `powershell`, `elvish`)

## Supported Shells

### Bash

Generate and install Bash completions:

```bash
# Generate completion script
swissarmyhammer completion bash > swissarmyhammer.bash

# Install for current user
mkdir -p ~/.local/share/bash-completion/completions
swissarmyhammer completion bash > ~/.local/share/bash-completion/completions/swissarmyhammer

# Or install system-wide (requires sudo)
sudo swissarmyhammer completion bash > /usr/share/bash-completion/completions/swissarmyhammer

# Source in current session
source ~/.local/share/bash-completion/completions/swissarmyhammer
```

Add to `~/.bashrc` for permanent installation:

```bash
# Add SwissArmyHammer completions
if [ -f ~/.local/share/bash-completion/completions/swissarmyhammer ]; then
    source ~/.local/share/bash-completion/completions/swissarmyhammer
fi
```

### Zsh

Generate and install Zsh completions:

```bash
# Generate completion script
swissarmyhammer completion zsh > _swissarmyhammer

# Install to Zsh completions directory
# First, add custom completion directory to fpath in ~/.zshrc:
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc

# Create directory and install
mkdir -p ~/.zsh/completions
swissarmyhammer completion zsh > ~/.zsh/completions/_swissarmyhammer

# Reload completions
autoload -U compinit && compinit
```

For Oh My Zsh users:

```bash
# Install to Oh My Zsh custom plugins
swissarmyhammer completion zsh > ~/.oh-my-zsh/custom/plugins/swissarmyhammer/_swissarmyhammer
```

### Fish

Generate and install Fish completions:

```bash
# Generate and install in one command
swissarmyhammer completion fish > ~/.config/fish/completions/swissarmyhammer.fish

# Completions are automatically loaded in new shells
```

### PowerShell

Generate and install PowerShell completions:

```powershell
# Generate completion script
swissarmyhammer completion powershell > SwissArmyHammer.ps1

# Add to PowerShell profile
Add-Content $PROFILE "`n. $(pwd)\SwissArmyHammer.ps1"

# Or install to modules directory
$modulePath = "$env:USERPROFILE\Documents\PowerShell\Modules\SwissArmyHammer"
New-Item -ItemType Directory -Force -Path $modulePath
swissarmyhammer completion powershell > "$modulePath\SwissArmyHammer.psm1"
```

### Elvish

Generate and install Elvish completions:

```elvish
# Generate and install
swissarmyhammer completion elvish > ~/.elvish/lib/swissarmyhammer.elv

# Add to rc.elv
echo "use swissarmyhammer" >> ~/.elvish/rc.elv
```

## What Gets Completed

The completion system provides intelligent suggestions for:

### Commands

```bash
swissarmyhammer <TAB>
# Suggests: serve, list, doctor, export, import, completion, config
```

### Command Options

```bash
swissarmyhammer serve --<TAB>
# Suggests: --port, --host, --debug, --watch, --prompts, etc.
```

### Prompt Names

```bash
swissarmyhammer get <TAB>
# Suggests available prompt names from your library
```

### File Paths

```bash
swissarmyhammer import <TAB>
# Suggests .tar.gz files in current directory

swissarmyhammer export output<TAB>
# Suggests: output.tar.gz
```

### Configuration Keys

```bash
swissarmyhammer config set <TAB>
# Suggests: server.port, server.host, prompts.directories, etc.
```

## Advanced Features

### Dynamic Completions

Some completions are generated dynamically based on context:

```bash
# Completes with actual prompt names from your library
swissarmyhammer get code-<TAB>
# Suggests: code-review, code-documentation, code-optimizer

# Completes with valid categories
swissarmyhammer list --category <TAB>
# Suggests: development, writing, data, productivity
```

### Nested Completions

Completions work with nested commands:

```bash
swissarmyhammer config <TAB>
# Suggests: get, set, list, validate

swissarmyhammer config set server.<TAB>
# Suggests: server.port, server.host, server.debug
```

### Alias Support

If you create shell aliases, completions still work:

```bash
# In .bashrc or .zshrc
alias sah='swissarmyhammer'

# Completions work with alias
sah serve --<TAB>
```

## Troubleshooting

### Completions Not Working

1. **Check Installation Location**
   ```bash
   # Bash
   ls ~/.local/share/bash-completion/completions/
   
   # Zsh
   echo $fpath
   ls ~/.zsh/completions/
   
   # Fish
   ls ~/.config/fish/completions/
   ```

2. **Reload Shell Configuration**
   ```bash
   # Bash
   source ~/.bashrc
   
   # Zsh
   source ~/.zshrc
   
   # Fish
   source ~/.config/fish/config.fish
   ```

3. **Check Completion System**
   ```bash
   # Bash
   complete -p | grep swissarmyhammer
   
   # Zsh
   print -l ${(ok)_comps} | grep swissarmyhammer
   ```

### Slow Completions

If completions are slow:

1. **Enable Caching** (Zsh)
   ```zsh
   # Add to ~/.zshrc
   zstyle ':completion:*' use-cache on
   zstyle ':completion:*' cache-path ~/.zsh/cache
   ```

2. **Reduce Dynamic Lookups**
   ```bash
   # Set static prompt directory
   export SWISSARMYHAMMER_PROMPTS_DIR=~/.swissarmyhammer/prompts
   ```

### Missing Completions

If some completions are missing:

```bash
# Regenerate completions after updates
swissarmyhammer completion bash > ~/.local/share/bash-completion/completions/swissarmyhammer

# Check SwissArmyHammer version
swissarmyhammer --version
```

## Environment Variables

Completions respect environment variables:

```bash
# Complete with custom prompt directories
export SWISSARMYHAMMER_PROMPTS_DIRECTORIES="/opt/prompts,~/my-prompts"
swissarmyhammer list <TAB>
```

## Integration with Tools

### fzf Integration

Combine with fzf for fuzzy completion:

```bash
# Add to .bashrc/.zshrc
_swissarmyhammer_fzf_complete() {
    swissarmyhammer list --format simple | fzf
}

# Use with Ctrl+T
bind '"\C-t": "$(_swissarmyhammer_fzf_complete)\e\C-e\er"'
```

### IDE Integration

Most IDEs can use shell completions:

#### VS Code
```json
{
    "terminal.integrated.shellIntegration.enabled": true,
    "terminal.integrated.shellIntegration.suggestEnabled": true
}
```

#### JetBrains IDEs
- Terminal automatically sources shell configuration
- Completions work in integrated terminal

## Custom Completions

### Adding Custom Completions

Create wrapper scripts with additional completions:

```bash
#!/bin/bash
# my-swissarmyhammer-completions.bash

# Source original completions
source ~/.local/share/bash-completion/completions/swissarmyhammer

# Add custom completions
_my_custom_prompts() {
    COMPREPLY=($(compgen -W "my-prompt-1 my-prompt-2 my-prompt-3" -- ${COMP_WORDS[COMP_CWORD]}))
}

# Override prompt name completion
complete -F _my_custom_prompts swissarmyhammer get
```

### Project-Specific Completions

Add project-specific completions:

```bash
# .envrc (direnv) or project script
_project_prompts() {
    ls ./prompts/*.md 2>/dev/null | xargs -n1 basename | sed 's/\.md$//'
}

# Export for use in completions
export SWISSARMYHAMMER_PROJECT_PROMPTS=$(_project_prompts)
```

## Best Practices

1. **Keep Completions Updated**
   ```bash
   # Update completions after SwissArmyHammer updates
   swissarmyhammer completion $(basename $SHELL) > ~/.local/share/completions/swissarmyhammer
   ```

2. **Test Completions**
   ```bash
   # Test completion generation
   swissarmyhammer completion bash | head -20
   ```

3. **Document Custom Completions**
   ```bash
   # Add comments in completion files
   # Custom completions for project XYZ
   # Generated: $(date)
   # Version: $(swissarmyhammer --version)
   ```

## Examples

### Complete Workflow

```bash
# Install completions
swissarmyhammer completion bash > ~/.local/share/bash-completion/completions/swissarmyhammer

# Use completions
swissarmyhammer li<TAB>          # Completes to: list
swissarmyhammer list --for<TAB>   # Completes to: --format
swissarmyhammer list --format j<TAB> # Completes to: json

# Get specific prompt
swissarmyhammer get code-r<TAB>   # Completes to: code-review

# Export with completion
swissarmyhammer export my-prompts<TAB> # Suggests: my-prompts.tar.gz
```

### Script Integration

```bash
#!/bin/bash
# setup-completions.sh

SHELL_NAME=$(basename "$SHELL")

case "$SHELL_NAME" in
    bash)
        COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
        ;;
    zsh)
        COMPLETION_DIR="$HOME/.zsh/completions"
        ;;
    fish)
        COMPLETION_DIR="$HOME/.config/fish/completions"
        ;;
    *)
        echo "Unsupported shell: $SHELL_NAME"
        exit 1
        ;;
esac

mkdir -p "$COMPLETION_DIR"
swissarmyhammer completion "$SHELL_NAME" > "$COMPLETION_DIR/swissarmyhammer"
echo "Completions installed to $COMPLETION_DIR"
```

## See Also

- [CLI Reference](./cli-reference.md) - Complete command documentation
- [Configuration](./configuration.md) - Configuration options
- [Getting Started](./getting-started.md) - Initial setup guide