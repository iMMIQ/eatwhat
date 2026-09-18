# Validation record

Verified on 2026-09-18 in [GitHub Actions run 35319087582](https://github.com/iMMIQ/eatwhat/actions/runs/35319087582).

Tested code commit: `9a4d97eff8e8287e51766ff6cd42d2d3560fb280`.

All 7 CI jobs passed:

- Rust: 12 integration tests, `cargo clippy --all-targets -- -D warnings`, and the Rust example.
- Python: native extension built and installed on Linux, macOS and Windows, each with Python 3.10 and 3.13.
- Each Python job ran 14 tests (7 catalog-tool tests and 7 binding tests), the catalog coverage command and the Python example.

The first run exposed a Windows default-encoding issue in a test fixture reader. The reader now explicitly uses UTF-8; the catalog CLI and example also emit UTF-8. The successful run above includes these fixes.

The bundled catalog contains 72 dishes in 9 categories, 70 time estimates, no complete ingredient lists and no geographic annotations. These are starter data, not an exhaustive or allergy-verified food database.

Local checks also passed: catalog tests, Python syntax checks and TOML parsing. Native compilation was verified on GitHub-hosted runners because the implementation workspace lacked a Rust toolchain.

No crates.io or PyPI release has been made. These tests do not establish compatibility with every Python/Rust version, CPU architecture or third-party catalog.
