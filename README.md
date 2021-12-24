# Embedded Interface Demo

A self-hosted web interface running on an ESP32-C3, in Rust. Starts a soft AP and web server on the device.

## Client

Requires [nodejs] and `npm` to build.

To install all dependencies and build the combined interface file, from within the `client/` directory:

```shell
$ npm install
$ npm run dev   # build for development
$ npm run prod  # build for production
$ npm run watch # watch for changes and automatically rebuild
```

To build the production version of the interface (ie. minified/gzipped) and place it in the correct location, from within the `client/` directory:

```shell
$ npm run prod
$ cp dist/index.html.gz ../server/resources/
```

[nodejs]: https://nodejs.org/en/

## Server

Requires the use of the Rust nightly release channel.

With the `index.html.gz` file already copied into the `resources/` directory, from within the `server/` directory:

```shell
$ cargo espflash --release --monitor
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
