import { extractTodos, convertNotesToTodos } from "./process"

describe('extractTodos', () => {
    test("empty string", () => {
        expect(extractTodos(``)).toStrictEqual([]);
    })

    test("ignores non TODOs1", () => {
        expect(extractTodos(`<ul><li>123</li><li>456</li></ul>`)).toStrictEqual([]);
    })

    test("works", () => {
        expect(extractTodos(`<div>#! TODO</div><ul><li>123</li><li>456</li></ul>`)).toStrictEqual(['123', '456']);
    })

    test("ignores non TODOs2", () => {
        expect(extractTodos(`<div>#! TODO</div>
        <ul>
        <li>123</li>
        <li>456</li>
        </ul>
        <div>Just a list</div>
        <ul>
        <li>789</li>
        <li>123</li>
        </ul>
        <ul>
        <li>123</li>
        <li>456</li>
        </ul>
    `)).toStrictEqual(['123', '456']);
    })

    test("more complicated", () => {
        expect(extractTodos(`<div>#! TODO</div>
        <ul>
        <li>123</li>
        <li>456</li>
        </ul>
        <ul>
        <li>789</li>
        <li>123</li>
        </ul>
        <div>#! TODO</div>
        <ul>
        <li>abc</li>
        <li>def</li>
        </ul>
    `)).toStrictEqual(['123', '456', 'abc', 'def']);
    })
});

describe('convertNotesToTodos', () => {
    test("convertNotesToTodos", () => {
        console.log(
            convertNotesToTodos([
                {
                    folder: "folder",
                    name: "9/16",
                    text: "text",
                    body: "<div>#! TODO</div><ul><li>1</li><li>2</li></ul>"
                },
                {
                    folder: "folder",
                    name: "10/16",
                    text: "text",
                    body: "<div>#! TODO</div><ul><li>3</li><li>4</li></ul>"
                }
            ])
        );
    })
});