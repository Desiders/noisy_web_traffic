[![MIT Licence](https://img.shields.io/pypi/l/aiogram.svg?style=flat-square)](https://opensource.org/licenses/MIT)

**noisy_web_traffic** is an application that allows you to create noise of web activity.

### Configuration
Configure the configuration for your own use in file `config.yaml`.

### How use
Before running an application, you can set up logger configuration (*optional*) in env using [env_logger](https://docs.rs/env_logger/latest/env_logger/)
(*check package's doc for more info*).

Linux:
- `export RUST_LOG=error|warn|info|debug|trace|off` (*optional*, default: *info*);<br>
- `./noisy_web_traffic` or
  `RUST_LOG=error|warn|info|debug|trace|off ./noisy_web_traffic` without 1 step;<br>

Windows:
- `set RUST_LOG=error|warn|info|debug|trace|off` (*optional*, default: *info*);<br>
- `./noisy_web_traffic.exe`;<br>

Docker:
- `docker run -e RUST_LOG=error|warn|info|debug|trace|off desiders/noisy_web_traffic` (*optional*, default: *info*);<br>
- `docker run desiders/noisy_web_traffic`;<br>

### [Releases](https://github.com/Desiders/noisy_web_traffic/releases)
