# Pure Provider Switch — agent instructions

- Read `SPEC.md` before changing behavior. Update it first when scope changes.
- Keep the app inside SPEC.md: URL/key switching, scoped link import, UI language and strictly user-triggered updates. Do not add other client-management features.
- Preserve user configuration and secrets. Never test against real client files.
- Use `rtk proxy` for shell commands. Read relevant source and consumers before edits.
- Follow build, validation and documentation ownership in `CONTRIBUTING.md`.
  Code changes require format, all-target Clippy, behavior tests and a release build;
  UI changes also require GUI acceptance. Documentation-only changes require
  link/anchor, Markdown, translation and evidence checks.
- Keep performance claims tied to measurements in `docs/verification.md`.
- Add tests for behavioral requirements, not for implementation details.
- Commit only task-owned changes. Do not rewrite history or discard others' edits.
