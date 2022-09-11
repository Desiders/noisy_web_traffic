[![MIT Licence](https://img.shields.io/pypi/l/aiogram.svg?style=flat-square)](https://opensource.org/licenses/MIT)

**noisy_web_traffic** is an application that allows you to create noise of web activity.

### Configuration
Configure the configuration for your own use in file `config.yaml`.

### How use
Before running an application, you can set up logger configuration (*optional*) in env using [env_logger](https://docs.rs/env_logger/latest/env_logger/)
(*check package's doc for more info*).

Docker:
- `docker pull desiders/noisy_web_traffic`;<br>
- `docker run desiders/noisy_web_traffic` or<br>
  `docker run -e RUST_LOG=debug desiders/noisy_web_traffic` with debug-level logs;<br>
- `docker stop <container_name>` to stop container.<br>

Linux:
- `export RUST_LOG=debug` if you need debug logs;<br>
- `./noisy_web_traffic` or `RUST_LOG=debug ./noisy_web_traffic` with debug-level logs (without 1 step);<br>
- `Ctrl + C` or `SIGINT` to exit the program.<br>

Windows:
- `set RUST_LOG=debug` if you need debug-level logs;<br>
- `./noisy_web_traffic`;<br>
- `Ctrl + C` or `SIGINT` to exit the program.<br>

### [Releases](https://github.com/Desiders/noisy_web_traffic/releases)
