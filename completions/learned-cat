
function _tests_list() {
    latest="${COMP_WORDS[$COMP_CWORD]}" 
    prev="${COMP_WORDS[$COMP_CWORD - 1]}"
    words=""
    
    if [[ $COMP_CWORD == 1 ]]; then
        words=`learned-cat -l | awk -F ' ' 'NR>2 {printf "%s ", $1}' && echo`
    fi

    COMPREPLY=($(compgen -W "$words" -- $latest))

    return 0
}

complete -F _tests_list learned-cat
