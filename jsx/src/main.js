import { config } from "./config"
import { getArgv, extractNotes, postTodos, getTodos, waitForDataTasks, pushReminder, setPassword } from "./clients";
import { convertNotesToTodos } from "./process";
import { parseArgv } from "./args" 

async function pushNotes() {
    console.log("push notes");
    const notes = extractNotes(config.includeFolders);
    const todos_entries = convertNotesToTodos(notes);
    
    console.log(
        JSON.stringify(todos_entries, null, 4)
    );

    await postTodos(config.userId, todos_entries);
    return "push success";
}

async function pullReminders(agentId = "agentId") {
    console.log("pull reminders");
    const todos = await getTodos(config.userId, agentId);
    console.log("get todos response", JSON.stringify(todos));

    todos.todos
    .map(todo => { return {
        name: todo.name,
        list: todo.list,
        some_date: "10/24"
    }})
    .forEach(pushReminder);

    return "pull success";
}

async function pushPull(agentId) {
    return await Promise.all([
        pushNotes(), 
        pullReminders(agentId)
    ]);
}

async function setPasswordCmd() {
    setPassword(config.userId, "123456");
}

const argv = getArgv();
const res = parseArgv(argv);
if (res === undefined) {
    throw "Usage: [push | pull]";
}

const command_map = {
    push: pushNotes,
    pull: pullReminders,
    paswd: setPasswordCmd,
    _default: pushPull,
};

const result_promise = command_map[res.command]();
result_promise.then(console.log);

waitForDataTasks();
