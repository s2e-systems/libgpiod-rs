# libgpiod in Rust

[![github](https://img.shields.io/badge/github-katyo/libgliod--rs-8da0cb.svg?style=for-the-badge&logo=github)](https://github.com/katyo/libgpiod-rs)
[![crate](https://img.shields.io/crates/v/libgpiod.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/libgpiod)
[![docs](https://img.shields.io/badge/docs.rs-libgpiod-66c2a5?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/libgpiod)
[![MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![CI](https://img.shields.io/github/workflow/status/katyo/libgpiod-rs/Rust?style=for-the-badge&logo=github-actions&logoColor=white)](https://github.com/katyo/libgpiod-rs/actions?query=workflow%3ARust)

Rust crate for interfacing with Linux GPIO character devices.

It provides an interface to the Linux GPIO using the chardev module.
This interface involves calling [ioctl](https://man7.org/linux/man-pages/man2/ioctl.2.html) funcions which are unsafe and require some unintuitive variable mapping.
To ease this process, this crate provides a [Chip] struct which encapsulates the interface in safe Rust functions. The functionality provided here is highly inspired by [libgpiod](https://git.kernel.org/pub/scm/libs/libgpiod/libgpiod.git/).

Since all functionality is dependent on Linux function calls, this crate only compiles for Linux systems.

## ABI compatibility

Both v1 and v2 ABI currently supported but edge detection implemented for v2 only.

## Async features

This crate support reading GPIO events asynchronously.
Current implementation depends from async runtime which is used.
The following integrations is supported:

- **tokio** for [tokio](https://tokio.rs/) fans
- **async-std** for [async-std](https://async.rs/) fans

Note that only one async feature can be selected.
