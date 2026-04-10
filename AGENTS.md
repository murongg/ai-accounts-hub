# Repository Agent Rules

- `docs/superpowers/` and `.superpowers/` are local planning artifacts.
- Never stage, commit, or push files from `docs/superpowers/` or `.superpowers/`, even with forced add commands such as `git add -f`.
- If a workflow asks for specs or plans in those directories, keep them local only and leave them out of git history.
- Do not change ignore rules just to make superpowers artifacts committable.
