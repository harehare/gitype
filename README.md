# gitype

Practice touch typing in the cli in your source code.

<img src="./demo.gif" alt="Demo">

Start typing by select a random file from the current or specified directory.
Files specified in .gitignore are excluded.

## Install

```bash
cargo install --git https://github.com/harehare/gitype.git
```

### manually
```
git clone https://github.com/harehare/gitype.git
cd gitype 
cargo run
```

## Usage

```bash
USAGE:
    gitype [OPTIONS]

OPTIONS:
    -d <dir>                       
    -e, --extension <EXTENSION>    
    -f <file>                      
    -h, --help                     Print help information
        --line <LINE>              [default: 20]
    -t <THEME>                     [default: dark]
        --time <TIME>              [default: 30]
    -V, --version                  Print version information
```

## License

[MIT](http://opensource.org/licenses/MIT)
