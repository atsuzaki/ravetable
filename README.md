## Ravetable
#### Rust + Wavetable = Ravetable 🦀

For CS 410P/510 Computers, Sound and Music (Spring 2021) at Portland State University

Katherine Philip

### Building
This program is written in rust, you'll need to install Rust in order to build it. The easiest way is with [Rustup](https://rustup.rs/).

On Linux, you may need these following packages:

- Cmake
- Mesa-libGL1 and Mesa-libGL-devel
- libX11-devel
- alsa-devel

Once you've obtained the rust toolchain and relevant packages, run it with `cargo run`.

### About Ravetable

Monophonic synth // <--- TODO

The synth may be played using your keyboard (key asedrhujik). To decrease and increase the octave, press Z and X.

There are 5 wavetable preset available, they are created in Serum's wavetable editor. Feature for loading custom wavetables will be implemented in the future.

### What I did, how it went & future work

Ravetable is built on top of an experimental branch of `tuix` with wgpu support and has been driving the development of said branch and `tuix` in general. Likewise, the advice from `tuix` collaborators have been of great help throughout the development of this project.


// TODO TODO TODO

The messaging bus and events are quite gnarly, still. I came across [this neat article by Devin Brite](https://dwbrite.com/blog/post/rust%20enums%20by%20example) about Rust enums which coincidentally talked about audio systems about two days before the project due date, and work on refactoring to enum-based system is still in progress. Not in love with the StatePacket scheme that is going on, though.
