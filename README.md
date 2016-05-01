`grow`
======

A growable pointer type for Rust. `Grow` generalises the idea of dynamically
allocated array-like types like `Vec` and `String` to arbitrary DSTs,
effectively eliminating the need for separate growable versions of types like
`[T]` and `str`.

`Grow<[T]>` is equivalent to `Vec<T>`, `Grow<str>` is equivalent to `String`,
`Grow<OsStr>` is equivalent to `OsString`, etc.. Note that none of the methods
like `push`, `pop` etc. are actually implemented on the corresponding `Grow`
types.

Usage
-----

Just add P1start/grow to your Cargo.toml:

```toml
[dependencies.grow]
git = "git://github.com/P1start/grow"
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
