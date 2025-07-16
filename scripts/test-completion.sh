#!/bin/bash

# Test completion functionality
# This script tests if the completion is working correctly

echo "Testing dbhub completion functionality..."

# Test if completion script generates without errors
echo "1. Testing completion script generation..."
if ./target/release/dbhub completion zsh > /dev/null 2>&1; then
    echo "   ✅ Zsh completion generation: OK"
else
    echo "   ❌ Zsh completion generation: FAILED"
    exit 1
fi

if ./target/release/dbhub completion bash > /dev/null 2>&1; then
    echo "   ✅ Bash completion generation: OK"
else
    echo "   ❌ Bash completion generation: FAILED"
    exit 1
fi

if ./target/release/dbhub completion fish > /dev/null 2>&1; then
    echo "   ✅ Fish completion generation: OK"
else
    echo "   ❌ Fish completion generation: FAILED"
    exit 1
fi

if ./target/release/dbhub completion powershell > /dev/null 2>&1; then
    echo "   ✅ PowerShell completion generation: OK"
else
    echo "   ❌ PowerShell completion generation: FAILED"
    exit 1
fi

# Test if help shows completion command
echo "2. Testing help output..."
if ./target/release/dbhub --help | grep -q "completion"; then
    echo "   ✅ Completion command in help: OK"
else
    echo "   ❌ Completion command in help: FAILED"
    exit 1
fi

echo ""
echo "🎉 All completion tests passed!"
echo ""
echo "To install completion for your shell, run:"
echo "  ./scripts/install-completion.sh zsh     # for zsh"
echo "  ./scripts/install-completion.sh bash    # for bash"
echo "  ./scripts/install-completion.sh fish    # for fish"
