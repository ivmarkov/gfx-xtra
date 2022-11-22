# gfx-xtra

[![CI](https://github.com/ivmarkov/gfx-xtra/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/gfx-xtra/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/v/gfx-xtra.svg)

Various add-ons to [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics). 
Currently, all of the add-ons are `DrawTarget` (display) transformations.

TL;DR: To use these, add one or both of the following to your Rust module:
```rust
use gfx_xtra::draw_target::DrawTargetExt2;
use gfx_xtra::draw_target::OwnedDrawTarget;
```

## MSRV

1.65, because the `Owned<...>` transformations use GATs, which just got stabilized.

## `PackedFrameBuffer`

An offscreen `DrawTarget` frame buffer implementation, with resolution of 1 to 8 bits per color. Used to implement flicker-free drawing and sending update deltas to the actual screen.

If you have 16 or 32bpp screen, use the `ColorConverted` `DrawTarget` transformation to convert your custom 1 to 8 bit color into the RGB color supported by your screen.
The 8 bit color restriction is unlikely to be lifted, as offscreen buffers with higher bpp require too much RAM.

## `Owned<...>` transformations

The `DrawTargetExt` trait in `embedded-graphics` allows you to clip, crop, translate and color-convert your display, 
but these transformations take a `&mut` reference to your original display, which sometimes can be inconvenient - as in when you want to transform your screen and
then send an owned (Box-ed or not) instance to a generic piece of drawing code.

Trait `OwnedDrawTargetExt` provides "owned" versions of these transformations, as well as of all transformations defined in this crate (rotated, scaled, buffered and flushing).

See [this embedded-graphics PR](https://github.com/embedded-graphics/embedded-graphics/pull/706) for more details.

## `Buffered` transformation

Uses two `PackedFrameBuffer` instances to achieve flicker-free incremental updates to the actual screen.

## Additional transformations

* `Rotated` - rotates the draw target to 90, 180 or 270 degrees
* (a bit controversial) `Scaled` - scales the draw target by a predefined ratio; makes sense for scaling down, not up
* Flushing - implements `Flushable` - an extension trait of `DrawTarget` that features a `flush` method. Useful when your display needs to be flushed at the end of the drawing, or when using a buffered transformation.

## Documentation, tests

None, as of now :p
