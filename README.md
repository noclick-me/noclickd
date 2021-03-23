# noclickd - noclick.me server

[![dependency status](https://deps.rs/repo/github/noclick-me/noclickd/status.svg)](https://deps.rs/repo/github/noclick-me/noclickd) [![Licenses Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fnoclick-me%2Fnoclickd.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fnoclick-me%2Fnoclickd?ref=badge_shield)

Share links with more descriptive URLs!

This repository is the home of the REST API server for the noclick.me, in
charge of expanding links retrieving information about them and creating
the more descriptive URLs for them.

If you are looking for a client, see
[noclick.me](https://github.com/noclick-me/noclick.me).

## License

The project is published under APGL (see [LICENSE.md](LICENSE.md)).

The main goal of choosing this license is to protect user's right. There is
a second goal, for which there are not very good known licenses, which is to
protect my ability (as developer behind the project) to sustain myself by
finding ways to make a living with my work.

Because of this I decided to use a standard open source license and wirte
a (non-legally binding) [declaration of
intent](https://github.com/llucax/llucax/blob/main/license-declaration-of-intent-v1.md).
Please read it if you want to make sure this software can stay alive and
healthy (specially if you plan to offer this software as a service).

## Contributing

This project is written in [Rust](https://https://www.rust-lang.org/). Once you have
a working rust toolchain installed, you can build it with `cargo build` and try
it out with `cargo run`.

To make logging more verbose, use `NOCLICKD_LOG=debug cargo run` (or `trace`
instead of `debug` for even more verboseness).
