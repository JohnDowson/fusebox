# fusebox

Mostly safe and sound append-only collection of trait objects.

## Why?
This avoids extra indirection of `Vec<dyn Trait>`, which might matter for you.
I personally use it in [pcmg](https://github.com/JohnDowson/pcmg) audio synthesizer for fusing together multiple filters and oscillators.

# Changelog

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
