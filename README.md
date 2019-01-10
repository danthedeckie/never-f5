# never-f5

I need to make a quick mockup of a site, or page/SPA/app concept.  Or I want to play with a new javascript library.

I don't want a node.js based server to get auto-refresh on save, or nice stuff like that.

I want it to be fast.  I want a stand-alone binary I can install anywhere.

(I want an excuse to learn rust...)

## Voila!

1) Serve a directory locally. (Port 8088)
2) Notify file changes by websocket
3) embed a websocket script into pages to auto refresh.

Makes quick page design / scripting a million times nicer. WIP.

You can access normal static files as normal:

    /index.html

for instance.  But if you add a `'!'`, then you get the auto-refresh websocket magic attached.

    /index.html!
    
Now, whenever any files in that directory are changed, it'll refresh the page.

## NEXT STEPS:

1) I don't actually want it to refresh on every file change - only on files that have been requested.  So Makefile changes, or source changes before they get compiled, and so on should be ignored.

2) I don't want it to refresh UNLESS YOU WANT IT TO.  So there should be a hook to allow it to call a user-defined 'file-changed' callback in the page.  This should allow you to save state (if you want to).

3) Make sure all the actix-web settings are tuned for this kind of work.  We aren't expecting a million requests and hundreds of simultanious connections.  - THAT SAID - It should allow you to have a bunch of different browsers, devices, etc. all viewing the same document.

4) Have a 'save-state/load-state' kind of route, so that you could make changes on one device, and see them reflected on multiple devices instantly - is this a good idea?

## Current State:

Very 'Work in Progress'.  I hacked this together very quickly, learning Rust and Actix-Web in the process.
It currently refreshes on every file change in the current directoy - which isn't ideal, but at least proves the concept.

So it's technically usable for me now, already, which is pretty awesome.

I put this on github now simply as an off-site backup from my laptop - and in case anyone else wants a similar tool.
