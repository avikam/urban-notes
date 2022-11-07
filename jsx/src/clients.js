import { config } from "./config"

ObjC.import('Cocoa')

const app = Application.currentApplication()
const session = $.NSURLSession;
const shared_session = session.sharedSession;

app.includeStandardAdditions = true;

export function extractNotes(includeFolders) {
    // use the notes app
    const notes_app = Application("notes");
    var collected = [];

    const folders = notes_app.folders;
    for (var fx = 0; fx < folders.length; fx++) {
        const folder = folders[fx];
        console.log("Inspecting folder", folder.name());

        if (!includeFolders.includes(folder.name())) {
            continue;
        }

        const notes = folder.notes;
        for (var nx = 0; nx < notes.length; nx++) {
            const note = notes[nx];

            collected.push(
                {
                    folder: folder.name(),
                    name: note.name(),
                    text: note.plaintext(),
                    body: note.body(),
                }
            )
        }
    }
    return collected;
}

export function pushReminder(todo) {
    const reminders_app = Application("Reminders");
    var new_reminder = reminders_app.Reminder({
        name: todo.todo, 
        body: `From: ${todo.folder} / ${todo.name}`
    });
    reminders_app.lists.byName("Reminders").reminders.push(new_reminder);
}

export function postTodos(note) {
    return request("POST", "push?user_id=user1", note);
}

export async function getTodos(user_id, agent_id) {
    return request("GET", `pull?user_id=${user_id}&agent_id=${agent_id}`);
}

function request(method, path, json_body) {
    const req =  $.NSMutableURLRequest.alloc.initWithURL($.NSURL.URLWithString(config.serverUrl + path));
    
    req.HTTPMethod = method;
    if (json_body) {
        req.HTTPContentType = "application/json;charset=utf-8";
        var body_string = $.NSString.alloc.initWithUTF8String(JSON.stringify(json_body));
        req.HTTPBody = body_string.dataUsingEncoding($.NSUTF8StringEncoding);
    }

    const promise = new Promise((resolve, reject) => {
        const task = shared_session.dataTaskWithRequestCompletionHandler(req, (data, resp, err) => {
            console.log(resp.statusCode);
            const raw_response = ObjC.unwrap($.NSString.alloc.initWithDataEncoding(data, $.NSASCIIStringEncoding));
            const parsed_response = JSON.parse(raw_response);

            resolve(parsed_response);
        });
        task.resume;
    });

    return promise;
}

// The code above schedules data task (maybe tasks in the future) that
// aren't guarenteed to be finished before the main function leaves.
// This function polls the currently running tasks while looping through runloops iterations.
// To keep the main runloop valid, there is an idle timer that is scheduled every second
// and the loop stops when no data tasks with completion handler exists.

// TODO: Can we simplfy this code with a simple sleep loop?
export function waitForDataTasks() {
    const interval = 1;
    const timer = $.NSTimer.timerWithTimeIntervalRepeatsBlock(interval, true, (_t) => {
        console.log("pending...");
    });

    const runloop = $.NSRunLoop.mainRunLoop;
    runloop.addTimerForMode(timer, $.NSDefaultRunLoopMode);

    while (!globalThis.__NoMoreData) {
        $.NSURLSession.sharedSession.getTasksWithCompletionHandler((data, upload, download) => {
            if (data.count === "0") {
                globalThis.__NoMoreData = true;
            }
        });

        runloop.runModeBeforeDate($.NSDefaultRunLoopMode, $.NSDate.dateWithTimeIntervalSinceNow(interval));
    }
}