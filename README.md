# calc_rs

[![Travis](https://img.shields.io/travis/cgm616/calc_rs/master.svg)](https://travis-ci.org/cgm616/calc_rs)
[![Website](https://img.shields.io/website-up-down-green-red/http/calc.cgm616.me.svg?label=demo)](http://calc.cgm616.me)
[![License](https://img.shields.io/badge/license-mit-blue.svg)](https://github.com/cgm616/calc_rs/blob/master/LICENSE)

`calc_rs` is a web-based, commandline-esque calculator built with [WebAssembly](http://webassembly.org/) and [Rust](https://www.rust-lang.org/en-US/).
Zero Javascript was written in the course of this project.
A demo is available here: [calc.cgm616.me](http://calc.cgm616.me).

## Why?

Truthfully, this project isn't very useful.
Instead, it's more of an exercise to see how complex of a web application can be built using only Rust, some html, and some css, and to see how easy/hard it is.
The intention was also to improve my html and css skills.

Overall, it was a great experience.
Stdweb and cargo-web make the Rust -> web process extremely easy, and WebAssembly itself improves web development for me by a couple orders of magnitude.
Bypassing the process of learning new tools and ecosystems, and what (to me) is the extremely confusing world of npm and Node, I was able to instead use Rust libraries to cover the functionality I wanted.
Instead of rolling my own parsing, I was able to use `pest`, which I had experimented with in the past.
There were no catches with using Rust crates, and it "just worked."

## Built with

`calc_rs` uses tools on the cutting edge of WebAssembly development.

- [cargo-web](https://github.com/koute/cargo-web), a commandline tool for compiling Rust to the client side web
- [stdweb](https://github.com/koute/stdweb), a standard library for the web
- [pest](https://github.com/pest-parser/pest), a PEG parser for Rust

## Building

In case you want to build `calc_rs` yourself, first install the nightly Rust compiler using [rustup](https://rustup.rs/).

```shell
> curl https://sh.rustup.rs -sSf | sh
> rustup default nightly
```

Then, install [cargo-web](https://github.com/koute/cargo-web), a tool for building client-side web applications with Rust.

```shell
> cargo install cargo-web
```

Finally, clone the project and build it.

```shell
> git clone https://github.com/cgm616/calc_rs
> cd calc_rs
> cargo web build --target=wasm32-unknown-unknown
```

A fully static site will be generated into the `./target/deploy` directory.
In case you want to develop `calc_rs`, it is easier to set up `cargo-web` to watch the directory and rebuild on changes.

```shell
> cargo web start --target=wasm32-unknown-unknown
```

A server will be running at `[::1]:8000` with calc_rs.
