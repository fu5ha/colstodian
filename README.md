# `colstodian`

[![crates.io](http://meritbadge.herokuapp.com/ultraviolet)](https://crates.io/crates/ultraviolet)
[![docs.rs](https://docs.rs/ultraviolet/badge.svg)](https://docs.rs/ultraviolet)

An opinionated color management library built on top of [`kolor`](https://docs.rs/kolor).

## Introduction

This library seeks to be a practical color management library for games and graphics.
It does so by encoding various information about a color either statically
in the Rust type system (as with the strongly-typed [`Color`]),
or data contained in the type (as with the dynamically-typed [`DynamicColor`]).
For more information, see [the docs](https://docs.rs/colstodian)
