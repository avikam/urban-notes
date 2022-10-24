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

### Running periodically

We want this utility to run periodically, so it will keep the Notes in sync with our server.
The preferred way to run background agents (over crontab) is using `launchd`.
We first write a configuration file, and place it and the built script in an accessible directory. I chose `~/Library/Scripts` for the script (without thinking too deeply about it) and `~/Library/LaunchAgents/` for the configuration. Then we execute some `launchctl` commands to bootstrap and enable the service.

```
# Copy script and launchd configuration (plist)
cp -r dist/urban_notes.app ~/Library/Scripts/urban_notes.app
cp com.urban_notes.agent.plist ~/Library/LaunchAgents/

# Install service
UID=$(id -u)
sudo launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.urban_notes.agent.plist
sudo launchctl enable gui/$UID/com.urban_notes.agent
```

### How I built it 

This code was bootstrapped thanks to the amazing wiki at the [JXA-Cookbook](https://github.com/JXA-Cookbook/JXA-Cookbook/wiki) repo.


Discovering the Notes API was easier with [debugging](https://developer.apple.com/library/archive/releasenotes/InterapplicationCommunication/RN-JavaScriptForAutomation/Articles/OSX10-11.html#//apple_ref/doc/uid/TP40014508-CH110-SW6) and inspecting, though it was quite predictable. 

The build pipeline transpile the ES6 files using babel and browserify into a single fat JXA file. It was tested on macOS Monterey.