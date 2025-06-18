
# mping - Multi-Host Ping Tool

[![Fair OSS User](https://img.shields.io/badge/Fair--OSS--User-%E2%9C%94-green)](https://yourproject.org/fair-use)

A concurrent ping utility written in Rust.

## Overview

**mping** is a command-line tool that can ping multiple hosts simultaneously
and display comprehensive statistics. Even though this tool solves a practical
problem I have had in the past, *it mainly is for me a practical learning exercise*
for exploring Rust's key features including async  programming, error handling,
and systems programming.

## Features

- **Concurrent Pinging**: Ping multiple hosts simultaneously using async/await.
- **Comprehensive Statistics**: Track sent/received packets, loss rates, and
  timing statistics.
- **Beautiful Output**: Formatted tables with UTF-8 borders using
  `comfy-table`.
- **Flexible Configuration**: Customizable ping interval and number of ICMP 
  packets to send.
- **Cross-Platform**: Works on macOS, Linux, and Windows.

## Usage

```bash
# Ping multiple hosts with default settings (5 pings each)
mping google.com 8.8.8.8 cloudflare.com

# Customize ping count and delay
mping -c 10 -d 0.5 example.com 1.1.1.1

# Ping with specific parameters
mping --count 20 --delay 1.5 github.com stackoverflow.com
```

**Note:** The minimum delay between packets *is 100 ms to avoid flooding multiple hosts
with ICMP packets*.  If you specify a smaller delay, it is automatically set
to 100 ms. For flood pinging I ask you to use the
[iputils ping](http://www.skbuff.net/iputils/) command.

## Sample Output

```
PING 3 hosts with 5 packets each in 1.00 s intervals ...
2a00:1450:4001:82f::200e ping error: Request timeout for icmp_seq 1

╭──────────────────────────────────────────────────────────────────────────────────────────────────╮
│ Host             Addr                       Sent   Recv   Loss    Min        Max        Avg      │
╞══════════════════════════════════════════════════════════════════════════════════════════════════╡
│ google.com       2a00:1450:4001:82f::200e   5      4      20.0%   21.60 ms   50.84 ms   34.42 ms │
│ -                8.8.8.8                    5      5      0.0%    24.16 ms   70.47 ms   43.10 ms │
│ cloudflare.com   2606:4700::6810:84e5       5      5      0.0%    30.04 ms   70.28 ms   44.15 ms │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯

Overall 15 sent, 14 received (6.67 % loss)
```

## Installation

### Prerequisites
- Rust 1.87.0 or later
- Cargo package manager

### Building from Source

```bash
git clone git@github.com:johndoe75/mping.git
cd mping
cargo build --release
```

### Running
``` bash
cargo run -- google.com 8.8.8.8
```

## Dependencies
- [**tokio**](https://github.com/tokio-rs/tokio): Async runtime for concurrent operations
- [**surge-ping**](https://github.com/kolapapa/surge-ping): ICMP ping implementation 
- [**clap**](https://github.com/clap-rs/clap): Command-line argument parsing 
- [**anyhow**](https://github.com/dtolnay/anyhow): Simplified error handling 
- [**comfy-table**](https://github.com/Nukesor/comfy-table): Beautiful table formatting 
- [**futures**](https://github.com/rust-lang/futures-rs): Async utilities
- [**colored**](https://github.com/colored-rs/colored): Terminal color output
- [**rand**](https://github.com/rust-random/rand): Random number generation

## Learning Resources
This project explores concepts covered in:
- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

## Contributing

This is primarily a learning project, but contributions that enhance the
educational value are welcome! Feel free to:

- Add comments explaining complex concepts
- Suggest alternative implementations
- Improve error handling patterns
- Add new features that demonstrate Rust concepts

# 🤝 Fair Use Policy

This project is licensed under the Apache 2.0 License and made available as
open source in the spirit
of collaboration and mutual benefit.

We kindly ask all users — especially commercial users — to follow this Fair
Use Policy:

### 💡 If you use this project in production:

- **Please contribute back** improvements, bug fixes, or enhancements whenever
  possible.
- **Please consider sponsoring** the project, supporting long-term maintenance
  and development.
- **Please open issues** if you encounter bugs or have ideas that could help
  others.

### 🔁 If you modify or extend the project:

- **Share your improvements** publicly, if possible.
- If not, **let the maintainers know privately** — we’re open to
  collaboration, even under NDA if needed.

### 🌱 If you benefit from the project:

- Give credit where due (e.g. in documentation or acknowledgments).
- Advocate for open source sustainability in your company or organization.

---

This is not a legal requirement — it's a social contract.

We believe in the open source ecosystem as a shared effort.  
If you benefit from this project, please be fair and help keep it alive.

Thank you for being a responsible open source user.

