# Pi-hole API

An HTTP API for Pi-hole.
The API reads FTL's shared memory so it can directly read the statistics FTL
generates. This API is the replacement for most of FTL's socket/telnet API, as
well as the PHP API of the old web interface.

## Getting Started (Development)

- Install Rust: https://www.rust-lang.org/tools/install
    - After installing, make sure the Rust tools are on your PATH:
      ```
      source ~/.cargo/env
      ```
- Install your distro's build tools
    - `build-essential` for Debian distros, `gcc-c++` and `make` for RHEL
      distros
- Install libsqlite3
    - `libsqlite3-dev` for Debian distros, `sqlite-devel` for RHEL
- Fork the repository and clone to your computer (not the Pi-hole). In
  production the Pi-hole only needs the compiled output of the project, not its
  source code
    - Checkout the `development` branch for the latest changes.
- Run `cargo check`. This will download project dependencies and check the
  program for errors. If everything was set up correctly, the final output
  should look like this:
  ```
      Finished dev [unoptimized + debuginfo] target(s) in 29.05s
  ```
- Run `cargo test`. This will compile and run the tests. They should all pass
  :wink:
- If you've never used Rust, you should look at the [documentation][Rust Docs],
  including the [Rust Book], before diving too deep into the code.
- When you are ready to make changes, make a branch off of `development` in your
  fork to work in. When you're ready to make a pull request, base the PR against
  `development`.

[Rust Docs]: https://www.rust-lang.org/learn
[Rust Book]: https://doc.rust-lang.org/book/
