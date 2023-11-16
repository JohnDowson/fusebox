# fusebox [![crates.io](https://img.shields.io/crates/v/fusebox.svg)](https://crates.io/crates/fusebox) [![CI status](https://img.shields.io/github/actions/workflow/status/JohnDowson/fusebox/rust.yml)](https://github.com/JohnDowson/fusebox/actions)
Mostly safe and sound append-only collection of trait objects.

## Why?
This avoids extra indirection of `Vec<Box<dyn Trait>>`, which might matter for you.
I personally use it in [pcmg](https://github.com/JohnDowson/pcmg) audio synthesizer for fusing together multiple filters and oscillators.

# Changelog
## 0.8.3
- Alignment bug in reallocation logic (#5)

## 0.8.2
- Fix bug in reallocation logic (#4)

## 0.8.0
- `push_unsafe` removed from public API
- `push` no longer requires `T: Send`, instead `Send` and `Sync` are implemented for `FuseBox<Dyn>` depending on wether `Dyn` is

## 0.7.0
- Improved iteration performance by using two pointer technique

## 0.6.0
- Performance improvements
- Soundness fixes

## 0.5.0
- Use `Unsize` instead of `AsDyn` marker trait, making safe push for foreign types possible

## 0.4.0
- Removed `Sz` parameter from `FuseBox`
- `FuseBox` now supports truly random access

## 0.3.0
- Added `Size` to restrict `Sz` to valid unsigned integers

## 0.2.0
- Added `AsDyn` to make safe `push` possible.
- Fixed pushed values not being dropped when `FuseBox` is dropped

## 0.1.0
Initial release
