import { extractNotes, postNotes } from "./clients";
import { convertNotesToTodos } from "./process";

const gIncludeFolders = [
    "Weekly goals"
]

function run() {
    const notes = extractNotes(gIncludeFolders);
    const todos_entries = convertNotesToTodos(notes);
    
    console.log(
        JSON.stringify(todos_entries, null, 4)
    );

    postNotes(todos_entries);

    // while (true) {
    delay(1);
    // }
    // var inp = app.doShellScript(`sleep 2`);
    // console.log(inp);
}

run();
console.log("done");
