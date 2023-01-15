# fusebox

Mostly safe and sound append-only collection of trait objects.

## Why?
This avoids extra indirection of `Vec<dyn Trait>`, which might matter for you.
I personally use it in [pcmg](https://github.com/JohnDowson/pcmg) audio synthesizer for fusing together multiple filters and oscillators.
