#!/bin/bash

# Install completion script for dbhub
# Usage: ./scripts/install-completion.sh [shell]
# Supported shells: zsh, bash, fish, powershell

set -e

SHELL_TYPE=${1:-zsh}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Build the project if binary doesn't exist
if [ ! -f "$PROJECT_DIR/target/release/dbhub" ]; then
    echo "Building dbhub..."
    cd "$PROJECT_DIR"
    cargo build --release
fi

case "$SHELL_TYPE" in
    zsh)
        # Create completion directory if it doesn't exist
        COMPLETION_DIR="$HOME/.zsh/completions"
        mkdir -p "$COMPLETION_DIR"
        
        # Generate and install enhanced completion script for zsh
        echo "Installing enhanced zsh completion for dbhub..."
        "$SCRIPT_DIR/generate-zsh-completion.sh" > "$COMPLETION_DIR/_dbhub"
        
        # Add to fpath if not already there
        ZSHRC="$HOME/.zshrc"
        if ! grep -q "/.zsh/completions" "$ZSHRC" 2>/dev/null; then
            echo "" >> "$ZSHRC"
            echo "# dbhub completion" >> "$ZSHRC"
            echo "fpath=(~/.zsh/completions \$fpath)" >> "$ZSHRC"
            echo "autoload -U compinit && compinit" >> "$ZSHRC"
        fi
        
        echo "✅ Enhanced Zsh completion with dynamic alias completion installed successfully!"
        echo "Please restart your shell or run: source ~/.zshrc"
        ;;
    bash)
        # For bash, we typically install to /usr/local/etc/bash_completion.d/ or ~/.bash_completion.d/
        COMPLETION_DIR="$HOME/.bash_completion.d"
        mkdir -p "$COMPLETION_DIR"
        
        echo "Installing bash completion for dbhub..."
        "$PROJECT_DIR/target/release/dbhub" completion bash > "$COMPLETION_DIR/dbhub"
        
        # Add to .bashrc if not already there
        BASHRC="$HOME/.bashrc"
        if ! grep -q ".bash_completion.d" "$BASHRC" 2>/dev/null; then
            echo "" >> "$BASHRC"
            echo "# dbhub completion" >> "$BASHRC"
            echo "for f in ~/.bash_completion.d/*; do source \$f; done" >> "$BASHRC"
        fi
        
        echo "✅ Bash completion installed successfully!"
        echo "Please restart your shell or run: source ~/.bashrc"
        ;;
    fish)
        # Fish completions go to ~/.config/fish/completions/
        COMPLETION_DIR="$HOME/.config/fish/completions"
        mkdir -p "$COMPLETION_DIR"
        
        echo "Installing fish completion for dbhub..."
        "$PROJECT_DIR/target/release/dbhub" completion fish > "$COMPLETION_DIR/dbhub.fish"
        
        echo "✅ Fish completion installed successfully!"
        echo "Fish will automatically load the completion on next startup."
        ;;
    powershell)
        echo "Installing PowerShell completion for dbhub..."
        "$PROJECT_DIR/target/release/dbhub" completion powershell
        echo ""
        echo "To install PowerShell completion, add the above output to your PowerShell profile."
        echo "You can find your profile location by running: \$PROFILE"
        ;;
    *)
        echo "Unsupported shell: $SHELL_TYPE"
        echo "Supported shells: zsh, bash, fish, powershell"
        exit 1
        ;;
esac
