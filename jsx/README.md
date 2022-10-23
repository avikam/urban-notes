## JXA automation for Notes. 

### What it does

The core functionality reads the content of some specified Notes (according to the currently hardcoded configuration). It then parses the HTML body of these notes and searches for lists that are marked with the `#! TODO` marker.

For example, a note that contains:
```
#! TODO
- item 1
- item 2
```

is actually serialized as a `<div>` followed by a `<ul>` list. It will extract the entries and post `item1, item2` to the server.

To run it, simply hit:

`npm run clean-run`

### How I built it 

This code was bootstrapped thanks to the amazing wiki at the [JXA-Cookbook](https://github.com/JXA-Cookbook/JXA-Cookbook/wiki) repo.


Discovering the Notes API was easier with [debugging](https://developer.apple.com/library/archive/releasenotes/InterapplicationCommunication/RN-JavaScriptForAutomation/Articles/OSX10-11.html#//apple_ref/doc/uid/TP40014508-CH110-SW6) and inspecting, though it was quite predictable. 

The build pipeline transpile the ES6 files using babel and browserify into a single fat JXA file. It was tested on macOS Monterey.