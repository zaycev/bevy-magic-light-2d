<div align="center">

# 🔮 Magic Light 2D – experimental dynamic 2D global illumination system for Bevy Engine.

[![Build status](https://github.com/zaycev/bevy-magic-light-2d/actions/workflows/pr.yml/badge.svg?branch=main)](https://github.com/zaycev/bevy-magic-light-2d/actions)
[![dependency status](https://deps.rs/repo/github/zaycev/bevy-magic-light-2d/status.svg)](https://deps.rs/repo/github/zaycev/bevy-magic-light-2d)

</div>

<div alight="center">

[![Discord](https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0b5061df29d55a92d945_full_logo_blurple_RGB.svg)](https://discord.com/invite/tuXBTxF3W2) (ping me if it expires)

</div>

2D Lighting Engine for Bevy supporting GI and other features.

## Examples

```shell
cargo run --example minimal
cargo run --example krypta
cargo run --example movement
```

## Contributing

- Code style:
```shell
rustup toolchain install nightly
cargo +nightly fmt
```

![Magic Light 2D – Demo](https://github.com/zaycev/bevy-magic-light-2d/blob/main/static/demo.gif?raw=true)

### Usage

### License

### TODOs

- [] Detect resolution
- [] Create SDF target
- [] Implement SDF pipeline
- [] Add debug HUD



```
Copyright 2022-2024 Vladimir Zaytsev <vladimir@xyzw.io>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
