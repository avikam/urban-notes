const yargsParser = require('yargs-parser')

const CMDS = {
    push,
    pull
}

export function parseArgv(argv) {
    var res = yargsParser(argv);
    var {'_': positionals, ...res} = res;
    if (positionals.length != 1 || CMDS[positionals[0]] === undefined) {
        return undefined
    }
    
    const [cmd, ] = positionals;
    return {
        command: cmd, 
        args: CMDS[cmd](res)
    }
}

function push(res) {
    return res
}

function pull(res) {
    return res
}
