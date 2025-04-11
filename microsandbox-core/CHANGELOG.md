# Changelog

## [0.2.1](https://github.com/microsandbox/microsandbox/compare/microsandbox-v0.2.0...microsandbox-v0.2.1) (2024-12-11)

### Features

- add log to cli and make examples work ([59d552a](https://github.com/microsandbox/microsandbox/commit/59d552a3b4e2a07936d101c425a90421593212e9))

## [0.2.0](https://github.com/microsandbox/microsandbox/compare/microsandbox-v0.1.0...microsandbox-v0.2.0) (2024-12-10)

### ⚠ BREAKING CHANGES

- **config:** Removes service type variants (Default, HttpHandler, Precursor) in favor of a single unified Service struct. This change simplifies the configuration model while maintaining all functionality.

### Code Refactoring

- **config:** simplify service model and improve volume handling ([#69](https://github.com/microsandbox/microsandbox/issues/69)) ([571219d](https://github.com/microsandbox/microsandbox/commit/571219da112ff484e8d0f77162e6d8704dc99f5f))

## [0.1.0](https://github.com/microsandbox/microsandbox/compare/microsandbox-v0.1.0...microsandbox-v0.1.0) (2024-12-03)

### Features

- add basic vm implementation ([#9](https://github.com/microsandbox/microsandbox/issues/9)) ([001f173](https://github.com/microsandbox/microsandbox/commit/001f173f80be2e63503222adb1f91bf61123bfeb))
- Add Linux overlayfs support and improve process monitoring ([#48](https://github.com/microsandbox/microsandbox/issues/48)) ([8329019](https://github.com/microsandbox/microsandbox/commit/832901982b6f788b5426f5f1c1055713a9c4b6e6))
- add monobase compiler module ([#43](https://github.com/microsandbox/microsandbox/issues/43)) ([da53a8b](https://github.com/microsandbox/microsandbox/commit/da53a8ba9adda7b503c9cde0c9eb5f3b7a8f064f))
- add primitives for supporting versioning ([#11](https://github.com/microsandbox/microsandbox/issues/11)) ([b13fb99](https://github.com/microsandbox/microsandbox/commit/b13fb9995e16c1a63f35f1d6a64742cc26aa28e2))
- Add REST API server mode ([#38](https://github.com/microsandbox/microsandbox/issues/38)) ([a84956d](https://github.com/microsandbox/microsandbox/commit/a84956d7b5a5e30dcaef78faa1ffd7d8520f035c))
- add subprojects ([#1](https://github.com/microsandbox/microsandbox/issues/1)) ([65e5744](https://github.com/microsandbox/microsandbox/commit/65e5744e11f5e061a567676d9a4d3ae25d3011c3))
- basic filesystem impl and other deps ([#10](https://github.com/microsandbox/microsandbox/issues/10)) ([83eee43](https://github.com/microsandbox/microsandbox/commit/83eee439166cad0c05cee569da6a417e47038f23))
- basic orchestration ([#20](https://github.com/microsandbox/microsandbox/issues/20)) ([3a01895](https://github.com/microsandbox/microsandbox/commit/3a0189560d6d7b61c114d482723185031f647e0f))
- build script for libkrun ([#7](https://github.com/microsandbox/microsandbox/issues/7)) ([dc33da5](https://github.com/microsandbox/microsandbox/commit/dc33da50e786db7bd71607960d831c208514220d))
- **ci:** add release automation workflow and dependency updates ([#62](https://github.com/microsandbox/microsandbox/issues/62)) ([1af4b4a](https://github.com/microsandbox/microsandbox/commit/1af4b4abf1ca90ec20738a72f0b8aca207acbaaa))
- download image layers and artifacts in appropriate folders ([#26](https://github.com/microsandbox/microsandbox/issues/26)) ([04444f9](https://github.com/microsandbox/microsandbox/commit/04444f9f6cf4144bda9e6b0cc0ab2d94a2290ddb))
- Implement CLI, service rootfs isolation and OCI image… ([#35](https://github.com/microsandbox/microsandbox/issues/35)) ([bd4fc2d](https://github.com/microsandbox/microsandbox/commit/bd4fc2dd7e07b2120c74000ea348c1880d4fad80))
- Improve README structure and enhance Makefile ([#37](https://github.com/microsandbox/microsandbox/issues/37)) ([8696096](https://github.com/microsandbox/microsandbox/commit/869609639fad91c76948d04508502881d2ae58ad))
- merge pulled container layers into a single rootfs dir ([#28](https://github.com/microsandbox/microsandbox/issues/28)) ([900d7e3](https://github.com/microsandbox/microsandbox/commit/900d7e3f29299c2218ad5c46af4f1de0cf1e690b))
- microsandbox config and project structure ([#4](https://github.com/microsandbox/microsandbox/issues/4)) ([cc44f06](https://github.com/microsandbox/microsandbox/commit/cc44f06eb7eea5784508bf35cf3d3cf21c8724c9))
- supervisor polls microvm cpu and mem usage ([#25](https://github.com/microsandbox/microsandbox/issues/25)) ([7298b30](https://github.com/microsandbox/microsandbox/commit/7298b305bd152d75ae91cc35eeab8f187d451262))
- support downloading Docker images ([#5](https://github.com/microsandbox/microsandbox/issues/5)) ([d727d7e](https://github.com/microsandbox/microsandbox/commit/d727d7e37bca5ab9b4153aaf5c3ced350e3605f1))

### Bug Fixes

- downloaded layer integrity checks ([#6](https://github.com/microsandbox/microsandbox/issues/6)) ([4231ede](https://github.com/microsandbox/microsandbox/commit/4231ede61bea7b6d773a9943af6726348cfa2ebc))
- Release-Please fix ([#66](https://github.com/microsandbox/microsandbox/issues/66)) ([2ba6e77](https://github.com/microsandbox/microsandbox/commit/2ba6e77d50db32abe1dc966a8d0ad4458fe871b6))
- Return symlink_metadata ([#49](https://github.com/microsandbox/microsandbox/issues/49)) ([b9be2c7](https://github.com/microsandbox/microsandbox/commit/b9be2c7ef5d4be33e282fe7681540daf8d3a9151))
