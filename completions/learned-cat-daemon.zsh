
function _tests_list() {
    latest="${COMP_WORDS[$COMP_CWORD]}" 
    prev="${COMP_WORDS[$COMP_CWORD - 1]}"
    words=""
    
    if [[ $COMP_CWORD == 1 ]]; then
        words="run export-marks export-variants help"
    else
        case "$prev" in
            export-marks)
                words=`ls *.csv`
                ;;
            export-variants)
                words=`ls *.csv`
                ;;
            *)
                ;;
        esac
    fi

    COMPREPLY=($(compgen -W "$words" -- $latest))

    return 0
}

complete -F _tests_list learned-cat-daemon
