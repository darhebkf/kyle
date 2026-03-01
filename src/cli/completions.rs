use anyhow::Result;

pub fn run(shell: &str) -> Result<()> {
    let script = match shell {
        "bash" => BASH_COMPLETION,
        "zsh" => ZSH_COMPLETION,
        "fish" => FISH_COMPLETION,
        _ => anyhow::bail!("Unsupported shell: {shell}. Use bash, zsh, or fish."),
    };

    print!("{script}");
    Ok(())
}

const BASH_COMPLETION: &str = r#"_kyle() {
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local commands="init config version upgrade mcp completions help"
    local global_flags="-v --version -h --help"

    case "${prev}" in
        config)
            COMPREPLY=($(compgen -W "list get set path" -- "${cur}"))
            return 0
            ;;
        completions)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "${cur}"))
            return 0
            ;;
    esac

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        local tasks=$(kyle --summary 2>/dev/null)
        COMPREPLY=($(compgen -W "${commands} ${tasks} ${global_flags}" -- "${cur}"))
        return 0
    fi
}

complete -F _kyle kyle
"#;

const ZSH_COMPLETION: &str = r#"#compdef kyle

_kyle() {
    local -a commands tasks

    commands=(
        'init:Create a new Kylefile'
        'config:Configure kyle settings'
        'version:Print version'
        'upgrade:Upgrade kyle to the latest version'
        'completions:Generate shell completions'
        'help:Print help'
    )

    if (( CURRENT == 2 )); then
        tasks=(${(f)"$(kyle --summary 2>/dev/null)"})
        _describe 'command' commands
        _describe 'task' tasks
    else
        case "${words[2]}" in
            config)
                local -a config_cmds=(
                    'list:Show all settings'
                    'get:Get a config value'
                    'set:Set a config value'
                    'path:Show config file path'
                )
                _describe 'config command' config_cmds
                ;;
            completions)
                _describe 'shell' '(bash zsh fish)'
                ;;
        esac
    fi
}

_kyle "$@"
"#;

const FISH_COMPLETION: &str = r#"function __kyle_tasks
    kyle --summary 2>/dev/null
end

function __kyle_needs_command
    set -l cmd (commandline -opc)
    test (count $cmd) -eq 1
end

function __kyle_using_command
    set -l cmd (commandline -opc)
    test (count $cmd) -gt 1; and test $cmd[2] = $argv[1]
end

complete -c kyle -n __kyle_needs_command -a '(__kyle_tasks)' -d 'task'
complete -c kyle -n __kyle_needs_command -a init -d 'Create a new Kylefile'
complete -c kyle -n __kyle_needs_command -a config -d 'Configure kyle settings'
complete -c kyle -n __kyle_needs_command -a version -d 'Print version'
complete -c kyle -n __kyle_needs_command -a upgrade -d 'Upgrade kyle to the latest version'
complete -c kyle -n __kyle_needs_command -a completions -d 'Generate shell completions'
complete -c kyle -n __kyle_needs_command -a help -d 'Print help'

complete -c kyle -n '__kyle_using_command config' -a 'list get set path'
complete -c kyle -n '__kyle_using_command completions' -a 'bash zsh fish'

complete -c kyle -s v -l version -d 'Print version'
complete -c kyle -s h -l help -d 'Print help'
"#;
