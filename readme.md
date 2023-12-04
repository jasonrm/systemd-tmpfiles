## About

A limited and simple implementation of `systemd-tmpfiles` in Rust.

Only supports the `d` type in configuration files for creation of directories.

User, group, and mode are not set/supported. Neither is cleaning up of files.

`--prefix <prefix>` is supported, and the main use case I have where I'm taking a `tmpfiles.conf` file from a NixOS derivation and want to create the paths outside systemd (in my case on macOS).

## Reference

- https://www.man7.org/linux/man-pages/man8/systemd-tmpfiles.8.html
- https://www.man7.org/linux/man-pages/man5/tmpfiles.d.5.html