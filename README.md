# `aoc2023` - My solutions to [Advent of Code 2023][aoc2023-website]

This repo contains my solutions to the [Advent of Code 2023][aoc2023-website]
challenges. They're implemented in Rust, within the framework of a highly
over-complicated TUI app built with [`ratatui`][ratatui], following the general
principles of The Elm Architecture and utilizing asynchronous processing.

And yes, I did start very late. This is what happens when your job is not programming.

## App architecture inspirations

The following projects were instrumental in helping me piece together this
(overly complex) app:

- The [`ratatui` async template][ratatui-async-template].
- [`rust-chat-server`'s TUI client][rust-chat-server-tui].
- [`elm-arch-examples-rs`][elm-arch-examples-rs].

[aoc2023-website]: https://adventofcode.com/2023
[ratatui]: https://ratatui.rs
[ratatui-async-template]: https://github.com/ratatui-org/templates/tree/main/async
[rust-chat-server-tui]: https://github.com/Yengas/rust-chat-server/tree/main/tui
[elm-arch-examples-rs]: https://github.com/boxdot/elm-arch-examples-rs
