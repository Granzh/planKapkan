# plan Kapkan

## Acknowledgements

This project stands on the shoulders of giants. Special thanks to the creators of:

- **[rust-tdlib](https://github.com/vhaoran/rust-tdlib)** -  Included as a subdirectory with minor modifications to better fit our project ecosystem.

- **[PyMax](https://github.com/MaxApiTeam/PyMax)** - Used as a core reference for architecture and logic for work with Max API, fully rewritten in Rust.

Both upstream libraries are licensed under the terms of the MIT License.

## About project

Репозиторий является backend частью проекта "Plan Kapkan", суть его заключается в том, чтобы использовать API Max для отправки и получения сообщений в Telegram. Проект на стадии разработки

## Prerequisites

- Rust toolchain (stable, `x86_64-pc-windows-msvc` target on Windows)
- **TDLib 1.8+** compiled for Windows (x64) — the `tdlib/` folder in this repo already contains the prebuilt binaries (see below). If you are linux or macOS enjoyer, you can build TDLib yourself and copy the resulting `tdjson.so` and its dependencies to `tdlib/`.
- A Telegram account with API credentials from <https://my.telegram.org>
- A Max/VK account

## Setup

### 1. Clone with submodules

```
git clone <this-repo-url>
cd plan_kapkan
```

> **MaxApiKernel** lives one level up (`../MaxApiKernel`). Clone it there:
> ```
> git clone https://github.com/Granzh/MaxApiKernel ../MaxApiKernel
> ```

### 2. Configure environment

```
cp .env.example .env
```

Edit `.env` and fill in:

| Variable              | Description                                                |
|-----------------------|------------------------------------------------------------|
| `API_ID`              | Telegram API ID from <https://my.telegram.org>             |
| `API_HASH`            | Telegram API hash from <https://my.telegram.org>           |
| `MAX_PHONE`           | Phone number for your Max/VK account (e.g. `+79001234567`) |
| `TDLIB_LOG_VERBOSITY` | Optional, default `1` (errors only)                        |

### 3. TDLib binaries

The `tdlib/` directory contains prebuilt Windows x64 binaries:

- `tdjson.dll` / `tdjson.lib` — TDLib itself
- `libssl-4-x64.dll`, `libcrypto-4-x64.dll` — OpenSSL (required by TDLib)
- `zlib1.dll` — zlib (required by TDLib)

`build.rs` automatically copies the DLLs next to the compiled executable and tells the linker where to find `tdjson.lib` — no manual steps required.

If you want to build TDLib yourself, replace the files in `tdlib/` with your build output and the matching OpenSSL/zlib DLLs.

### 4. Build and run

```
cargo run
```

On first launch TDLib will prompt you to enter your Telegram phone number and the confirmation code. Credentials are cached in `tdlib-data/` for subsequent runs.

Max will also ask for a confirmation code on first run (printed to stdout).

## How it works

```
Telegram Saved Messages  ←→  [plan_kapkan bridge]  ←→  Max Saved Messages
```

- Incoming Max messages (`chat_id = 0`) are forwarded to your Telegram Saved Messages chat.
- Incoming Telegram messages in Saved Messages are forwarded to Max Saved Messages.
- Echo suppression prevents infinite loops: a message sent by the bridge is ignored when it bounces back.

## Notes

- Session data is stored in `tdlib-data/` (TDLib) — add this to `.gitignore`.
- Max session cookies/tokens are managed by MaxApiKernel internally.
- Non-text content (stickers, photos, etc.) is silently ignored by the bridge.
