import { config } from "./config"
import { extractNotes, postNotes, pushReminder } from "./clients";
import { convertNotesToTodos } from "./process";

function pushNotes() {
    const notes = extractNotes(config.includeFolders);
    const todos_entries = convertNotesToTodos(notes);
    
    console.log(
        JSON.stringify(todos_entries, null, 4)
    );

    postNotes(todos_entries);
}

function pullReminders() {
    // GET cursor time
    // GET todos { thisTime }

    pushReminder({
        todo: "Planning of a new sprint",
        folder: "Weekly goals",
        name: "10/24"
    });

    // POST done { thisTime }
}

function main() {
    pushNotes();

    pullReminders();
}

main();
console.log("done");
