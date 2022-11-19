import { sign } from "./sign"
import { config } from "./config"

ObjC.import('Cocoa')
ObjC.import('stdlib')

const app = Application.currentApplication()
app.includeStandardAdditions = true;

const session = $.NSURLSession;
const shared_session = session.sharedSession;

export function extractNotes(includeFolders) {
    const now = new Date();
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
            const modified = note.modificationDate();
            const since_days = Math.ceil((now - modified) / (1000 * 60 * 60 * 24));
            if (since_days > config.notesSince) {
                continue;
            }

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

export function postTodos(tokenAccountName, note) {
    const password = getPassword(tokenAccountName);
    return request(tokenAccountName, password, "POST", "push", "", note);
}

export async function getTodos(tokenAccountName, agent_id) {
    const password = getPassword(tokenAccountName);
    return request(tokenAccountName, password, "GET", "pull", `agent_id=${agent_id}`);
}

function request(tokenAccountName, password, method, path, query, json_body) {
    const req =  $.NSMutableURLRequest.alloc.initWithURL($.NSURL.URLWithString(`${config.serverUrl}/${path}?${query}`));
    
    req.HTTPMethod = method;
    if (json_body) {
        req.HTTPContentType = "application/json;charset=utf-8";
        var body_string = $.NSString.alloc.initWithUTF8String(JSON.stringify(json_body));
        req.HTTPBody = body_string.dataUsingEncoding($.NSUTF8StringEncoding);
    }
    const { signature, nonce, account, timestamp } = sign(tokenAccountName, password, query);
    req.setValueForHTTPHeaderField($(`${nonce}.${timestamp}.${account}.${signature}`), $('Authorization'));

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

export function setPassword(username, password) {
    $.system(
        `/usr/bin/security add-generic-password -a '${username}' -s 'urbannotes.com' -w '${password}' -U`
    );
}

function getPassword(username) {
    return app.doShellScript(`/usr/bin/security find-generic-password -a '${username}' -s 'urbannotes.com' -w`);
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