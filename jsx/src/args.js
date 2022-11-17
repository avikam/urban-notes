const yargsParser = require('yargs-parser')

const CMDS = {
    push,
    pull,
    paswd
}

export function parseArgv(argv) {
    var res = yargsParser(argv);
    var {'_': positionals, ...rest} = res;

    if (positionals.length > 1 ) {
        return undefined
    }
    
    const [cmd, ] = positionals;
    
    if (cmd !== undefined && CMDS[cmd] === undefined) {
        return undefined
    }

    const args_function = CMDS[cmd] || _default;
    return {
        command: cmd || '_default', 
        args: args_function(rest)
    }
}

function _default(res) {
    return res
}

function push(res) {
    return res
}

function pull(res) {
    return res
}

function paswd(res) {
    return res
}
