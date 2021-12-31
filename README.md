## Asar Extract

### Why?
Because I wanted something else than node-js to extract files from an asar-archive.

### Warning
Code is ugly as hell as I'm not used to write Rust. It was a project to start at least once a year something in this language.

I'm sure there are bugs included as well.

### How to run the binary
```
asar_extract FILE [DEST_DIR]
```
If no *DEST_DIR* is provided it will extract it into the current directory.
