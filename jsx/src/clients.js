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

export function postNotes(note) {
    return request("POST", "push?user_id=user1", note);
}

export function pullReminders(cursor) {
    return request("GET", `pull?user_id=user1&agent_id=agent1`);
}

function request(method, path, json_body) {
    const req =  $.NSMutableURLRequest.alloc.initWithURL($.NSURL.URLWithString(config.serverUrl + path));
    
    req.HTTPMethod = method;

    if (json_body) {
        req.HTTPContentType = "application/json;charset=utf-8";
        var body_string = $.NSString.alloc.initWithUTF8String(JSON.stringify(json_body));
        req.HTTPBody = body_string.dataUsingEncoding($.NSUTF8StringEncoding);
    }

    shared_session.dataTaskWithRequestCompletionHandler(req, (data, resp, err) => {
        console.log(resp.statusCode);
        console.log(ObjC.unwrap($.NSString.alloc.initWithDataEncoding(data, $.NSASCIIStringEncoding)));
    }).resume;
}
