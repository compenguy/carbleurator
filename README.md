# Carbleurator

`carbleurator` is a project to control the Elegoo Smart Robot Car kit using a
USB gamepad and BLE.

The built application should work from any Windows, Linux, or macOS system
equipped with BLE-enabled bluetooth.

# OpenEmbedded integration
[picard](https://github.com/compenguy/picard) is an OpenEmbedded-based linux
distribution tailored to running an application such as carbleurator on a
raspberry pi.

To build this component with openembedded and the meta-rust layer, first
generate a bitbake recipe with:

```bash
$ cargo install cargo-bitbake
$ cargo bitbake
```

Copy the resulting recipe into a recipes folder in your project layer,
and add the meta-rust layer to the project.

Test building the package with:

```bash
$ bitbake carbleurator
```
