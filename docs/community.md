# Community research behind “Why pp-switch?”

Reviewed public cc-switch issues and pull requests on 2026-10-12. These are
individual requests and reports, not a representative survey, a benchmark,
or evidence that every current cc-switch version has a particular defect.
Nothing here implies endorsement of pp-switch.

| Source | What the author said | Product implication |
| --- | --- | --- |
| [#7649: Pure / Lite mode](https://github.com/farion1231/cc-switch/issues/7649) | “对于只希望使用 CC Switch 来「切换供应商 / API / 模型」的用户来说，管理范围有些过大。” The request also says: “CC Switch only manages fields it owns. Everything else belongs to the target application/user.” | Make ownership explicit. pp-switch owns only effective URL/key values; it deliberately excludes the request's broader model, routing and usage scope. |
| [#4142](https://github.com/farion1231/cc-switch/issues/4142) | A Windows user on cc-switch 3.16.2 reported losing Codex `config.toml` settings after toggling local routing, then needing to redo sandbox/project trust settings. | Preserve unrelated configuration bytes and refuse unsupported setups. This is a version-specific report, not a claim about current cc-switch. |
| [#4581](https://github.com/farion1231/cc-switch/issues/4581) | A macOS user on 3.16.3 reported WebContent memory growth during prolonged residence, saying they usually opened the tool only when needed and sometimes forgot to exit it. | Close the window → exit the process. Keep measurements honest: pp-switch's macOS RSS also exceeds its own budget. This report is not a comparative benchmark. |
| [PR #4693](https://github.com/farion1231/cc-switch/pull/4693) | Proposed destroying the WebView when entering tray mode. Open and unmerged when reviewed. | The upstream community is addressing lifecycle costs too. Do not present an open proposal as a released fix or an ongoing defect in all versions. |

The README leads with the explicit Lite-mode request, then describes pp-switch's
own verified behavior. It does not advertise that pp-switch is faster than
cc-switch: we have not run a controlled comparison. Config-preservation tests
and process-exit behavior are our evidence; performance budgets and failures
remain in [verification.md](verification.md).

cc-switch is a broader management application. Its additional capabilities are
useful when you need them; pp-switch serves the narrower “my setup already works,
just change URL and key” workflow. Maintain that distinction in release notes
and future promotional copy.
