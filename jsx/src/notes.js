const cheerio = require('cheerio');

const app = Application.currentApplication()
ObjC.import('Cocoa')
const session = $.NSURLSession;
const shared_session = session.sharedSession;

app.includeStandardAdditions = true;

const include_list = [
    "1-1 notes",
    "9/26"
]

function extract_notes() {
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

function post_notes(note) {
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

/**
 * Process a note object, return a list of TODO items
 * @param {*} note 
 * @returns [text, text, ...]
 */
function process_notes(note) { 
    const $ = cheerio.load(note.body);
    return $('li').toArray().map(li => cheerio.text($(li)));
}

var notes = extract_notes();
notes.forEach(note => {
    var extended_note = {
        ...note,
        todos: process_notes(note)
    }
    console.log(JSON.stringify(extended_note, null, 4));
    // post_notes(note);
});

// 
// post_notes(
//             {
//                 folder: "folder",
//                 name: "note.name()",
//                 text: "note.plaintext()",
//                 body: "note.body()",
//             }
// );

function run() {
    // while (true) {
    delay(1);
    // }
    // var inp = app.doShellScript(`sleep 2`);
    // console.log(inp);
}

run();
console.log("done");
