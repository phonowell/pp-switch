# pp-switch contributor instructions

- Read `SPEC.md` before changing behavior. Update it first when scope changes.
- Keep the app inside SPEC.md: URL/key switching, scoped link import, UI language and strictly user-triggered updates. Do not add other client-management features.
- Preserve user configuration and secrets. Never test against real client files.
- Use `rtk proxy` for shell commands. Read relevant source and consumers before edits.
- Validate with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` and a release build. Exercise the GUI after UI changes.
- Keep performance claims tied to measurements in `docs/verification.md`.
- Add tests for behavioral requirements, not for implementation details.
- Commit only task-owned changes. Do not rewrite history or discard others' edits.
