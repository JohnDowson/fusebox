# fusebox

Mostly safe and sound append-only collection of trait objects.

## Why?
This avoids extra indirection of `Vec<dyn Trait>`, which might matter for you.
I personally use it in [pcmg](https://github.com/JohnDowson/pcmg) audio synthesizer for fusing together multiple filters and oscillators.

# Changelog

## 0.2.0
- Added `AsDyn` to make safe `push` possible.
- Fixed pushed values not being dropped when `FuseBox` is dropped

## 0.1.0
Initial release
