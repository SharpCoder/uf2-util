# uf2-util

`uf2-util` is a small utility for generating .uf2 files. This program was designed specifically for the rp2040
and as such it assumes you have two separate binaries. A bootloader that is around 252 bytes, and then some 
much larger third-stage binary.

It will automatically calculate and incorporate the bootloader crc checksum into the generated uf2.

## Installation

Use the cargo package manager to install uf2-util.

```bash
cargo install uf2-util
```

## Usage

```bash
uf2-util --bootrom bootloader.bin --progdata app.bin --out app.uf2
```


## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)