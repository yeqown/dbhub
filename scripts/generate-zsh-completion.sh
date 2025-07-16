#!/bin/bash

# Generate enhanced zsh completion script for dbhub with dynamic alias completion
# This script creates a zsh completion that can dynamically load database aliases

cat << 'EOF'
#compdef dbhub

autoload -U is-at-least

_dbhub() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
        '-h[Print help]' \
        '--help[Print help]' \
        '-V[Print version]' \
        '--version[Print version]' \
        ":: :_dbhub_commands" \
        "*::: :->dbhub" \
        && ret=0
    case $state in
    (dbhub)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:dbhub-command-$line[1]:"
        case $line[1] in
            (connect|c)
                _arguments "${_arguments_options[@]}" \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    ':alias:_dbhub_aliases' \
                    && ret=0
                ;;
            (context|e)
                _arguments "${_arguments_options[@]}" \
                    '--generate[Generate default config file]' \
                    '--filter-env=[Environment name]:FILTER_ENV: ' \
                    '--filter-db-type=[Database type]:FILTER_DB_TYPE: ' \
                    '--filter-alias=[Alias name]:FILTER_ALIAS: ' \
                    '--with-dsn[Output format control: with_dsn]' \
                    '--with-annotations[Output format control: with_annotations]' \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    && ret=0
                ;;
            (completion|comp)
                _arguments "${_arguments_options[@]}" \
                    '-h[Print help]' \
                    '--help[Print help]' \
                    ':shell:(zsh bash fish powershell)' \
                    && ret=0
                ;;
            (help)
                _arguments "${_arguments_options[@]}" \
                    '*::subcommand -- The subcommand whose help message to display:_dbhub_commands' \
                    && ret=0
                ;;
        esac
        ;;
    esac
}

(( $+functions[_dbhub_commands] )) ||
_dbhub_commands() {
    local commands; commands=(
        'connect:Connect to a database using environment and database name'
        'c:Connect to a database using environment and database name'
        'context:Manage database connection contexts'
        'e:Manage database connection contexts'
        'completion:Generate shell completion scripts'
        'comp:Generate shell completion scripts'
        'help:Print this message or the help of the given subcommand(s)'
    )
    _describe -t commands 'dbhub commands' commands "$@"
}

(( $+functions[_dbhub_aliases] )) ||
_dbhub_aliases() {
    local aliases
    # Get aliases from dbhub configuration
    aliases=($(dbhub completion-suggestions aliases 2>/dev/null))
    if [[ ${#aliases[@]} -gt 0 ]]; then
        _describe -t aliases 'database aliases' aliases
    else
        # Fallback to file completion if no aliases found
        _files
    fi
}

if [[ $zsh_eval_context == *loadautofunc* ]]; then
    # autoload context, do nothing
    _dbhub "$@"
else
    # eval context, call function directly
    _dbhub "$@"
fi
EOF
