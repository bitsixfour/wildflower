# wildflower
NEED TO KNOW!
 * Not going to support fading audio/cross-fade
 * 500 album limit
 * Can't create custom playlists
 * No tag system YET
 * no tag system, but most mpd clients don't use this (ncncmpp, rmpc)
 * Although, it *will* read your servers reqwests but can't create new ones because MPD doesn't have a specfic
 plugin for that 

Simple POC for an MPD server that simply instead of reading your local fs... reads your navidrome server.....\
Instead of using SQLite or whatever, it makes a simple text file from what subsonic supports. This is why this is a personal POC.
