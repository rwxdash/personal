# Notes — Learning Tutorials

This repo contains **zero-to-advanced learning tutorials** for technologies and languages.
Each topic lives in its own directory (e.g., `c-notes/`).

## How to generate a new tutorial

When the user asks to learn a topic, create a self-contained directory: `<topic>-notes/`

### Directory structure

```
<topic>-notes/
  .vscode/
    settings.json       # language-specific editor settings
    extensions.json     # official extensions only
  README.md             # overview table of lessons, build/run instructions, how-to guide
  Makefile (or equivalent build file)
  01-<topic>.ext        # numbered lesson files, standalone, heavily annotated
  02-<topic>.ext
  ...
```

### Lesson file conventions

- **Numbered sequentially**: `01-`, `02-`, ... — zero-padded, ordered basic to advanced.
- **Standalone**: each file compiles/runs on its own (unless explicitly multi-file like headers).
- **Heavily annotated**: comments explain *everything* — the what, why, and gotchas.
- **Comparisons**: if the user knows another language, map concepts to that language (e.g., Rust types vs C types).
- **Exercises at the bottom**: each lesson ends with hands-on exercises the user can try immediately.
- **Break things**: encourage the user to intentionally break code to understand failure modes.
- **Build command in header**: each file includes its own compile/run command in a top comment.

### README.md format

Follow the pattern in `c-notes/README.md`:
- One-line description of the tutorial's goal
- Table of all lessons (number, file, topic)
- Build & run commands
- "How to work through them" section

### Build system

- Provide a `Makefile` (or language-appropriate equivalent: `Cargo.toml`, `package.json`, `go.mod`, etc.)
- Support: build all, build one, run one, clean
- For C/C++: generate `compile_commands.json` and `.clangd` for editor support
- For interpreted languages: a run script or task runner

### VSCode configuration

**Rules:**
- Use **official extensions only** (Microsoft, language maintainers)
- Check existing projects for reference configs:
  - C/C++: see `c-notes/.vscode/`
  - Rust: use `rust-lang.rust-analyzer`, `tamasfe.even-better-toml` (ref: `../../projects/mici/.vscode/`)
  - For other languages: pick the official/canonical extension

**`extensions.json`**: list recommended extensions for the language.
**`settings.json`**: disable conflicting features, set language-specific preferences.

### Formatter / linter configs

Include language-appropriate config files in the topic directory:
- C/C++: `.clang-format`, `.clangd`
- Rust: `rustfmt.toml` if needed (rust-analyzer handles most)
- Python: `pyproject.toml` or `.ruff.toml`
- Go: gofmt is standard, no config needed
- JS/TS: `.prettierrc` or similar

### .gitignore

The root `.gitignore` covers:
- `/tmp` — scratch directory
- `*.o`, `*.out` — C build artifacts
- `.cache`, `.clangd`, `compile_commands.json` — clangd artifacts

When adding a new language, update the root `.gitignore` if the language produces new artifact types (e.g., `target/` for Rust, `node_modules/` for JS).

### Lesson progression philosophy

**Ask the user about their familiarity level first.** The starting point depends on context:

- **Unfamiliar language**: start from basics (types, variables, hello world) and build up to advanced topics.
- **Familiar language**: skip basics entirely. Focus on idiomatic patterns, design patterns, advanced features, or specific domains the user wants to deepen.
- **Specific goal** (e.g., eBPF, async patterns, embedded): tailor the curriculum toward that goal — only cover foundational material if it's needed to get there.

In all cases:
1. Each lesson builds on the previous but is readable in isolation
2. Final lessons connect to the user's actual goal if one is stated
3. Match the depth to the user's level — don't waste time on what they already know

### Scope

- Generate **all lesson files in one go** — the user should have a complete curriculum to work through
- Aim for 10-15 lessons for a typical topic (adjust based on complexity)
- Each file should be 80-200 lines — dense enough to be useful, short enough to digest
