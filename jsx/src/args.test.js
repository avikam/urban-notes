import { parseArgv } from "./args"

describe('extractTodos', () => {
    test("sanity commands", () => {
        expect(parseArgv([])).toStrictEqual(undefined);
        expect(parseArgv(['pusx'])).toStrictEqual(undefined);
        expect(parseArgv(['pul'])).toStrictEqual(undefined);
    })

    test("sanity commands", () => {
        expect(parseArgv(['push', '--since', '60'])).toStrictEqual({command: 'push', args: {since: 60}});
    });

    test("sanity commands", () => {
        expect(parseArgv(['pull', '--since', '60'])).toStrictEqual({command: 'pull', args: {since: 60}});
    });
});
