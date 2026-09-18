# Validation record

Checked in the implementation workspace on 2026-09-18:

- PASS: 7 Python catalog-tool tests (validity, exact duplicate merge, conflicting merge, alias collisions, unknown sources, missing knowledge and numeric bounds).
- PASS: bundled catalog coverage check: 72 dishes, 9 primary categories, 70 time estimates, 0 complete ingredient lists, 0 geographic annotations.
- PASS: Python syntax compilation for the wrapper, examples/tools/tests as applicable.
- PASS: TOML parsing for Cargo.toml and pyproject.toml.

Not executed locally:

- Rust compilation, cargo test and clippy.
- Building a Python native wheel and running binding integration tests.

The workspace has no Rust toolchain. The system package installation failed on container permissions; the official rustup download timed out at the network proxy. No successful compile or Rust test result is claimed.

`.github/workflows/ci.yml` defines these remaining checks for GitHub Actions. A passing CI run is required before treating the native extension as verified. No crates.io or PyPI release has been made.
