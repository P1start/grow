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
