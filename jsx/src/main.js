import { config } from "./config"
import { extractNotes, postTodos, getTodos, waitForDataTasks, pushReminder } from "./clients";
import { convertNotesToTodos } from "./process";
import { parseArgv } from "./args"

async function pushNotes(userId) {
    const notes = extractNotes(config.includeFolders);
    const todos_entries = convertNotesToTodos(notes);
    
    console.log(
        JSON.stringify(todos_entries, null, 4)
    );

    postNotes(todos_entries);
}

async function pullReminders(userId, agentId) {
    console.log("pull reminders");
    const todos = await getTodos("user1", "agentId");
    console.log("get todos response", todos);

    todos
    .map(todo => { return {
        todo,
        folder: "Weekly goals",
        name: "10/24"
    }})
    .forEach(pushReminder);

    return true;
}

globalThis.runMain = (argv) => {
    const res = parseArgv(argv);
    if (res === undefined) {
        throw "Usage: [push | pull]";
    }

    const command_map = {
        push: pushNotes,
        pull: pullReminders
    };

    const result_promise = command_map[res.command]();
    result_promise.then(console.log);

    waitForDataTasks();
}
