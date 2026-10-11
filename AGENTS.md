# pp-switch contributor instructions

- Read `SPEC.md` before changing behavior. Update it first when scope changes.
- Keep the app inside its spec: only provider URL/key switching and link import.
- Preserve user configuration and secrets. Never test against real client files.
- Use `rtk proxy` for shell commands. Read relevant source and consumers before edits.
- Validate with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` and a release build. Exercise the GUI after UI changes.
- Keep performance claims tied to measurements in `docs/verification.md`.
- Add tests for behavioral requirements, not for implementation details.
- Commit only task-owned changes. Do not rewrite history or discard others' edits.
