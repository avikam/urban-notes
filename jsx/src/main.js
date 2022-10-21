import { extractNotes } from "./notes";
import { extractTodos } from "./parse";


var notes = extractNotes();
notes.forEach(note => {
    var extended_note = {
        ...note,
        todos: extractTodos(note.body)
    };

    console.log(
        JSON.stringify(extended_note.todos, null, 4)
    );
    // post_notes(note);
});

// 
// post_notes(
//             {
//                 folder: "folder",
//                 name: "note.name()",
//                 text: "note.plaintext()",
//                 body: "note.body()",
//             }
// );

function run() {
    // while (true) {
    delay(1);
    // }
    // var inp = app.doShellScript(`sleep 2`);
    // console.log(inp);
}

run();
console.log("done");
