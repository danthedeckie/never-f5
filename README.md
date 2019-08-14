# never-f5

I need to make a quick mockup of a site, or page/SPA/app concept.

Or work with a designer quickly iterating HTML changes to a design, and the
"save", "switch to a browser window", "press f5" loop is getting pretty old.

I want auto refreshing, fast, without needing to install a massive node.js
multi-layer complex beast.

I want it to be fast.  I want a stand-alone binary I can install anywhere.

(I want an excuse to code in rust...)

## ta DAA! Here is "never-f5" to save the day.

1) Serve a directory locally. (Port 8088)
2) Notify file changes by websocket
3) embed a websocket script into pages to auto refresh.

Makes quick page design / scripting a million times nicer.

You can access normal static files as normal:

    /index.html

for instance.  But if you add a `'!'`, then you get the auto-refresh websocket
magic attached.

    /index.html!

Now, whenever any files in that directory are changed, it'll refresh the page.

## NEXT STEPS:

0) *More Commandline arguments.*
   Need to have directory, websocket route address, doing clever CSS stuff, etc.

1) *Tidying up* - general fixing / cleaning / organising / documenting / refactoring.

2) Have a `save-state/load-state` kind of route, so that you could make changes
on one device, and see them reflected on multiple devices instantly - /is this a
good idea?/

## Current State:

I put this together pretty quickly, learning Rust and Actix-Web in the
process.  It seems to work very well.

## Saving state.

Since it's sometimes nice to have state in web apps or pages (who knew, right)
there's a couple of callbacks you can add to your page: `window._autoreload_save`
and `window._autoreload_load`, which you can use to save and reload your state
to local storage or whatever before and after reloads.  If you want to do more
complex stuff, you can specify your own javascript file to append on to `!` files.
