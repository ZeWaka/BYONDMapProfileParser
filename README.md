# byond map-profile parser

This tool is a utility which takes in SendMaps profiling logs from
a BYOND server and graphs all of the various entries.

You can compile it yourself, but windows builds can also be found on the [releases page].

[releases page]: https://github.com/ZeWaka/byond_map-profile_parser/releases

## Dependencies

The [Rust] compiler:

1. Install the Rust compiler's dependencies (primarily the system linker):

   * Ubuntu: `sudo apt-get install gcc-multilib`
   * Windows (MSVC): [Build Tools for Visual Studio 2017][msvc]
   * Windows (GNU): No action required

1. Use [the Rust installer](https://rustup.rs/), or another Rust installation method,
   or run the following:

    ```sh
    curl https://sh.rustup.rs -sSfo rustup-init.sh
    chmod +x rustup-init.sh
    ./rustup-init.sh
    ```


## Compiling

The [Cargo] tool handles compilation, as well as automatically downloading and
compiling all Rust dependencies. To compile in release mode (recommended for
speed):

```sh
cargo build --release
# output: target/release/byond_map-profile_parser
```


## Usage
If you're using data from `514.1554`, the data is malformed. You'll need to
replace `"unit"` with `,"unit"`

First argument is to a folder containing your log files.

Example: `byond_map-profile_parser.exe examples`

If you're having problems, ask in the [Coderbus Discord] for ZeWaka.



[Rust]: https://rust-lang.org
[Cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/
[msvc]: https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=15
[Coderbus Discord]: https://discord.gg/Vh8TJp9

## License

This project is licensed under the [MIT license](https://en.wikipedia.org/wiki/MIT_License).

See [LICENSE](./LICENSE) for more details.
