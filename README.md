# CHIP-8 Interpreter

An interpreter for the CHIP-8 programming language.

## Features

* Prioritizes accuracy
* Interpreter runs in its own thread and sends frames to screen thread when rendering

## Usage

Run a CHIP-8 ROM using:

```bash
cargo run --release -- path/to/rom.ch8
```

## Building

To build the interpreter:

```bash
cargo build --release
```

## Contribution

I'm not looking for contributions, but if you find issues or have suggestions for improvements, feel free to open an issue or start a discussion.

## License

This project is released under The Unlicense, making it completely free to use, modify, and distribute without restrictions.

See the [UNLICENSE](./UNLICENSE) file for more details.

## References

* Technical Reference: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
* Wiki: https://en.wikipedia.org/wiki/CHIP-8
* Original Reference: https://pong-story.com/chip8/
* ROMS: https://www.zophar.net/pdroms/chip8.html
