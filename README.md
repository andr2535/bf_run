# Bf_run

Bf_run is a brainfuck interpreter and recompiler.  
For information about functionality of the program, run the program with "--help" flag

### Running

#### Running using Cargo

```
cargo run --release
```

### Installing

#### Install using deb file

There is a .deb file included in the releases tab of Github.  
It can be installed using this command:
```
dpkg -i debfilename.deb
```

#### Install using Cargo

```
cargo install --path .
```

#### Other OS than Linux

Bf_run has only been tested on Linux.
However I don't see any reason why it wouldn't work on Windows or Mac OS.

The recompiler will not work on any platform that is not x86-64 though.

## License

Bf_run is licensed under the GPLv3 [license](LICENSE) or later.