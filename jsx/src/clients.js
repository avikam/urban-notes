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

export function postNotes(note) {
    const req =  $.NSMutableURLRequest.alloc.initWithURL($.NSURL.URLWithString('http://127.0.0.1:8000/notes'));
    
    req.HTTPMethod = "post";
    req.HTTPContentType = "application/json;charset=utf-8";
    
    var body_string = $.NSString.alloc.initWithUTF8String(JSON.stringify(note));
    req.HTTPBody = body_string.dataUsingEncoding($.NSUTF8StringEncoding);

    shared_session.dataTaskWithRequestCompletionHandler(req, (data, resp, err) => {
        console.log(resp.statusCode);
        console.log(ObjC.unwrap($.NSString.alloc.initWithDataEncoding(data, $.NSASCIIStringEncoding)));
    }).resume;
}
