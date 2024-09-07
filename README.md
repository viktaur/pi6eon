# Pi6eon 🐦
Like a carrier pigeon over IPv6.

## Description
Pi6eon is a minimal, direct, stateless, and end-to-end encrypted CLI app that enables direct chatting between two IPv6 address over TCP.

Key exchange has been implemented using the [`x25519-dalek`](https://github.com/dalek-cryptography/curve25519-dalek/tree/main/x25519-dalek) crate. The two parties generate a shared secret that is then used for symmetric message encryption using the [`aes-gcm`](https://github.com/RustCrypto/AEADs/tree/master/aes-gcm) crate. Please note that, although these two crates have been externally audited, there are no safety guarantees regarding their implementation in this project. **⚠️ USE AT YOUR OWN RISK ⚠️**.

Finally, for communication to be possible, there needs to exist a network path free of any firewall or NAT restrictions between the two IPv6 addresses.

## Quick start
Ensure that you have the latest Rust version installed (if you don't have Rust installed, you can easily get it [here](https://www.rust-lang.org/tools/install))
```
rustup update
```

Clone the repository and execute
```
cargo build --release
```

To see the actions available
```
./target/release/pi6eon --help


Usage: pi6eon <COMMAND>

Commands:
  setup
  listen
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Screenshots
<img width="1512" alt="Screenshot 2024-09-07 at 14 25 07" src="https://github.com/user-attachments/assets/866b2330-6577-4c09-97d3-4314ba444545">
<img width="1363" alt="Screenshot 2024-09-07 at 14 25 25" src="https://github.com/user-attachments/assets/3a6ded3e-ec7f-4a1c-8682-9c1f0684310b">
