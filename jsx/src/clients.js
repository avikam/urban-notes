ObjC.import('Cocoa')

const app = Application.currentApplication()
const session = $.NSURLSession;
const shared_session = session.sharedSession;

app.includeStandardAdditions = true;

const include_list = [
    "1-1 notes",
    "9/26"
]

export function extractNotes() {
    // use the notes app
    const notes = Application("notes");
    var collected = [];

    // for (var f in notes.folders) {
    //     console.log(notes.folders[f].name());
    //     console.log(JSON.stringify(notes.folders[f].properties()));
    // }

    for (var i in notes.notes) {
        var note = notes.notes[i];
        if (!include_list.includes(note.name())) {
            continue;
        }

        // console.log(JSON.stringify(notes.notes[i].properties()));
        // console.log(note.body());

        collected.push(
            {
                folder: "TODO: folder",
                name: note.name(),
                text: note.plaintext(),
                body: note.body(),
            }
        )
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
