/* 
Parse Notes
We explicitly mark a list inside a note to be synced with a #! TODO text. 
*/

const cheerio = require('cheerio');

/**
 * Process a note object, returns a list of TODO items
 * @param {*} note 
 * @returns [text, text, ...]
 */
export const extractTodos = (note_body) => {
    const $ = cheerio.load(note_body);
    // Find lists that has an immediate DIV siblig (+ sign) with the #! TODO marker
    const todo_lists = $("div:contains('#! TODO') + ul");

    return todo_lists
        .children("li")
        .toArray().map(
            li => cheerio.text($(li))
        );
}

export function convertNotesToTodos(notes) {
    return notes.flatMap(note => {
        const {body, folder, name} = note;
        return extractTodos(body).map(todo => {
            return {todo, folder, name}
        })
    });
}