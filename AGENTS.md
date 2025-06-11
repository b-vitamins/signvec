# Development Guidelines for Agents

This repository contains a Rust crate. Contributions should maintain a consistent style and follow the rules outlined below.

## Commit Message Standards
- Use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
- Format: `type(scope): summary` e.g. `feat(parser): add new AST parser`.
- Common types: `feat`, `fix`, `docs`, `chore`, `refactor`, `perf`, `test`.
- Use the imperative mood in the summary line and keep it under 72 characters.

## Commit Sequencing
- Break changes into logical, atomic commits.
- Keep commits focused; unrelated changes require separate commits.
- Order commits so that tests and lints pass at each step.

## Pull Request Standards
- Title should summarise the change using Conventional Commit style.
- Description must include:
  - **Summary**: high level overview of changes.
  - **Testing**: commands run and their results.
  - **Future Work**: optional notes on follow ups.

## Code Housekeeping
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before committing.
- Remove unused code and keep dependencies up to date.
- Track technical debt in issue tracker when needed.

## Architecture and Design
- Document significant design decisions in `/docs/adr-<number>.md` using the [ADR format](https://adr.github.io/).
- Keep modules focused; avoid large files with mixed concerns.
- Public API changes require updates to documentation and examples.

## Versioning
- This crate follows [Semantic Versioning](https://semver.org/).
- Increment:
  - **MAJOR** for incompatible API changes.
  - **MINOR** for backwards compatible functionality.
  - **PATCH** for backwards compatible bug fixes.
- Tags are of the form `vX.Y.Z`.

## Changelog Maintenance
- Update `CHANGELOG.md` for any externally visible change.
- Use sections: `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.
- Keep an `Unreleased` section at the top.

## Testing Standards
- Prefer unit tests close to the code they test.
- Use doctests for examples as seen in `src/`.
- Ensure `cargo test` passes before pushing.

## Documentation Standards
- Keep `README.md` up to date with major changes.
- Public items must have rustdoc comments.
- Examples should compile as part of the test suite.

