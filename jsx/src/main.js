import { config } from "./config"
import { extractNotes, postNotes } from "./clients";
import { convertNotesToTodos } from "./process";

function run() {
    const notes = extractNotes(config.includeFolders);
    const todos_entries = convertNotesToTodos(notes);
    
    console.log(
        JSON.stringify(todos_entries, null, 4)
    );

    postNotes(todos_entries);
}

run();
console.log("done");
