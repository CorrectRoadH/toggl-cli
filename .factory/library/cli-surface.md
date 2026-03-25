# CLI Surface

Canonical command-surface guidance for this mission.

**What belongs here:** Resource names, action naming, compatibility boundaries, UX rules.
**What does NOT belong here:** Low-level implementation details already covered in `architecture.md`.

---

## Canonical grammar

Use:

```text
toggl <resource> <action> [args] [flags]
```

Resources for this mission:

- `auth`
- `entry`
- `project`
- `tag`
- `task`
- `client`
- `workspace`
- `org`
- `preferences`
- `config`

## UX rules

- Do not preserve the old verb-first interface as the primary command surface.
- Prefer explicit actions over overloaded implicit behavior.
- Keep argument names and JSON behavior predictable for both humans and AI agents.
- For commands that support structured output, `--json` must remain prose-free on stdout.

## Auth and local debug rules

- Local Cargo-driven debug should work with repo-local `.env`.
- Help/parser-only flows must not require credentials.
- OpenToggl/custom API URL flows are first-class, not secondary add-ons.
