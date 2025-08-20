# rlocate

rlocate is a personal project of mine to write a path keyword identifier for Linux-based systems. You can search for either a specific keyword in a path or the basename of a path and the program will search through all of the paths in your filesystem. 

**NOTE**: This program is still in testing, and some folders are excluded from database storage to increase speed of program. 
## How to Run

```shell
git clone https://github.com/dylannandlall/rlocate.git
cargo build --release
cd ./target/release
./rlocate 
```

To update the database at any time, run this:
```shell
./rlocate updatedb
```

To search paths in the database based off of keyword, use the `-k` command:
```shell
./rlocate -k [keyword]
```

To search paths in the database based off of the basename, use the `-b` command:
```
./rlocate -b [basename]
```