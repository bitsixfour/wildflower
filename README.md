
this is a small MPD-compatible server that presents a Subsonic library to MPD clients. It keeps a local SQLite snapshot for database querying, albeit I obviously haven't tested mega databases.\
I needed this because of a personal need to use MPD clients; I don't want to use samba or nfs lol.


This... 
- Fetches up to all albums and posts it onto sqlite
- supports core MPD features to listen to music (from my experience) but isn't exactly 1:1 
- uses rodio for playback


## try it

```sh
export NAVIDROME_URL=http://127.0.0.1:4533
export NAVIDROME_USER=your-user
export NAVIDROME_PASSWORD=your-password
cargo run --release
```


## USE IT!!!!


```nix
services.wildflower = {
  enable = true;
  url = "http://127.0.0.1:6767";
  usrname = "user";
  password = "password";
  port = 6600;
};
```


## other need to know
- no audio fade or cross-fade
- I don't cache album covers. Probably should get that done soon.
