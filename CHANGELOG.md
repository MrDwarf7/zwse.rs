# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-07-22

### Bug Fixes

- clippy clean under -D warnings (allow unsafe_code until stripped) - ([b44524d](https://github.com/MrDwarf7/zwse.rs/commit/b44524d5d26be2414ae5423d8a37755a0eaff8c4))
- use actions/checkout@v7 directly (centralized action can only run after checkout) - ([d76bb6d](https://github.com/MrDwarf7/zwse.rs/commit/d76bb6d6fb3e00198efafe8afd52a6760f5a8081))

### Documentation

- README banners use assets/ header + icon - ([00de886](https://github.com/MrDwarf7/zwse.rs/commit/00de886e1baa38e1ad6916077481f40ec28f5231))

### Features

- assets/ icons + Windows build.rs (winresource) - ([03b1275](https://github.com/MrDwarf7/zwse.rs/commit/03b12750b6b1145185089d3f16178f410cb01bcf))
- apply rust_template conventions (workflows, configs, README) - ([974ac51](https://github.com/MrDwarf7/zwse.rs/commit/974ac5171eda15eb24bd070fa2b6d77786387821))
- core extractor logic impl - ([7b43d46](https://github.com/MrDwarf7/zwse.rs/commit/7b43d4627d3f2dd54d85a2757e3518735d11d746))

### Miscellaneous Tasks

- **release:** bump to 0.1.0 - ([fc9ae3b](https://github.com/MrDwarf7/zwse.rs/commit/fc9ae3b5551511126425029b0cbb00da3f6b93f7))
- publish promotes draft via gh API (drop svenstaro force path) - ([8ea845b](https://github.com/MrDwarf7/zwse.rs/commit/8ea845b073dfd86ea66c1e6689966a0286ed781b))
- sync workflows to latest rust_template (publish.yml, no force-push nightly) - ([97f2c13](https://github.com/MrDwarf7/zwse.rs/commit/97f2c13e932d3a0bee7116923fa30c5f03785385))
- add .gitattributes, centralized checkout action - ([5812ffc](https://github.com/MrDwarf7/zwse.rs/commit/5812ffc98bbe4bff248dd73472f2b5b0b61cd902))
- sync workflows with latest rust_template (checkout@v7 centralized) - ([773ea14](https://github.com/MrDwarf7/zwse.rs/commit/773ea14682469e227874d1a7b3391304a2074ccc))
- untrack .extras/ (should be gitignored) - ([83a2179](https://github.com/MrDwarf7/zwse.rs/commit/83a2179c05736df1ee295c95ca038d879714aba0))
- add .github with issues, pr and workflows - ([5341b8d](https://github.com/MrDwarf7/zwse.rs/commit/5341b8dca8d2fc377d563eca2903ed069c332712))

### Styling

- cargo fmt - ([a15fafc](https://github.com/MrDwarf7/zwse.rs/commit/a15fafcae1603d0873eecc6b793864339b55af20))

### Testing

- lib split + unit/integration coverage for export pipeline - ([66f6eeb](https://github.com/MrDwarf7/zwse.rs/commit/66f6eebb6b4d818ab3f7ebac683b079326a575ec))

### Cd

- Makefile build system - ([bd5a2e7](https://github.com/MrDwarf7/zwse.rs/commit/bd5a2e74f6c0a8285bc0b643b2c290402d678202))


