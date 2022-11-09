import { config } from "./config"
import { extractNotes, postTodos, getTodos, waitForDataTasks, pushReminder } from "./clients";
import { convertNotesToTodos } from "./process";
import { parseArgv } from "./args"

async function pushNotes(userId) {
    console.log("push notes");
    const notes = extractNotes(config.includeFolders);
    const todos_entries = convertNotesToTodos(notes);
    
    console.log(
        JSON.stringify(todos_entries, null, 4)
    );

    await postTodos(todos_entries);
    return "push success";
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

    return "pull success";
}

async function pushPull(userId, agentId) {
    return await Promise.all([
        pushNotes(userId), 
        pullReminders(userId, agentId)
    ]);
}

globalThis.runMain = (argv) => {
    const res = parseArgv(argv);
    if (res === undefined) {
        throw "Usage: [push | pull]";
    }

    const command_map = {
        push: pushNotes,
        pull: pullReminders,
        _default: pushPull,
    };

    const result_promise = command_map[res.command]();
    result_promise.then(console.log);

    waitForDataTasks();
}
