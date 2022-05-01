placeviewer
===========

https://reddit.com/r/place

## Instructions

0. Clone and build with `cargo`.

1. Download 2017 and 2022 placement logs. They will need to be cleaned up to convert user ids to integers.

2. Run commands to convert data into a binary format. 
```
./target/release/placeviewer parse data/placements/2017-log.csv data/cache/2017 1000 1000 500
./target/release/placeviewer parse data/placements/2022-log.csv data/cache/2022 2000 2000 500
```

3. Start server with `./target/release/placeviewer serve config.yaml`. ports and host can be configured through command line args. Run `./target/release/placeviewer --help` for more options.  

## API

### `/images/{name}/tiles/{tile_x}/{tile_y}/ts/{timestamp}.png`
Get a tile at the specified timestamp for a dataset.
- name: name of dataset (eg 2017 or 2022)
- tile_x: x position of tile
- tile_y: y position of tile
- timestamp: unix timestamp in milliseconds

### `/images/{name}/tiles/{tile_x}/{tile_y}/diff-ts/{timestamp1}_{timestamp2}.png`
Generate a diff of a tile at two specific timestamps.
- name: name of dataset
- tile_x: x position of tile
- tile_y: y position of tile
- timestamp1: unix timestamp in milliseconds
- timestamp2: unix timestamp in milliseconds

### `/images/{name}/tiles/{tile_x}/{tile_y}/uid-rem/{user_id}_{timestamp}.png`
Get a user's surviving placements at a specific timestamp.
- name: name of dataset
- tile_x: x position of tile
- tile_y: y position of tile
- user_id: user id
- timestamp: unix timestamp in milliseconds


### `/images/{name}/tiles/{tile_x}/{tile_y}/uid/{user_id}.png`
Get all placements by a user. If a user has placed two placements on top of each other, only the most recent one will be returned in the image.
- name: name of dataset
- tile_x: x position of tile
- tile_y: y position of tile
- user_id: user id