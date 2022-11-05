# Urban Notes

This is a way to synchronize the action items scattered around my Notes with my Todo app (currently AnyDo). While that is the goal of this project, the main motivation to start it was to learn Rust.

There are two components:
- server: A Rust rocket server with a simple API that pushes Todos to my tasks app.
The server now requires a postgres to store the state of the synchronization.
- jxa: An AppleScript automation written in JavaScript, that runs in the background, extracts TODO items written in my notes, and posts them to the server.


## The name
The name was shuffled by GitHub when I started this repo.
