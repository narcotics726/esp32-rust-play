# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the entry point and wires the board, services, and device loops.
- `src/board.rs` initializes peripherals and holds device handles.
- `src/devices/` contains hardware drivers (e.g., `wifi.rs`, `oled.rs`, `button.rs`, `max98357a.rs`).
- `src/services/` contains runtime services like `http_server.rs`.
- `build.rs` loads `.env` values into `cargo:rustc-env` at build time.
- `sdkconfig.defaults` and `rust-toolchain.toml` define ESP-IDF defaults and the Rust toolchain channel.

## Build, Test, and Development Commands
- `cargo build` builds the firmware using the ESP toolchain.
- `cargo build --release` builds a size-optimized release (`opt-level = "s"`).
- `cargo run` flashes and runs if your ESP-IDF tooling is configured for `cargo run` on the target.
- `cargo fmt` formats code using `rustfmt.toml` settings.

## Coding Style & Naming Conventions
- Rust 2021 edition; keep line width at 80 and allow rustfmt to wrap comments.
- Use snake_case for modules/functions and CamelCase for types.
- Prefer short, descriptive names for devices and channels (`btn1`, `snd`, `rcv`).

## Testing Guidelines
- No automated tests are present. If you add tests, place unit tests in-module and name files `*_test.rs` for clarity.
- Run `cargo test` for host-side logic, but note ESP-IDF targets may not support tests on-device.

## Commit & Pull Request Guidelines
- Commit messages in history are short, imperative, and lowercase (e.g., `add wifi and http server`).
- Keep commits focused and scoped to one feature or fix.
- PRs should include a concise summary, hardware tested (board model), and any required `.env` changes.

## Configuration Tips
- `.env` is optional; add `wifi_ssid` and `wifi_psw` for local builds.
- Keep secrets out of Git and document required keys in the PR description.
