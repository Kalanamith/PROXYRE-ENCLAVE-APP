# Conventional Commit Format Guide

## Format: `type(scope): description`

## Common Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `build`: Changes that affect the build system or external dependencies
- `ci`: Changes to our CI configuration files and scripts
- `chore`: Other changes that don't modify src or test files

## Examples:

### For your case (dependency updates):
❌ Wrong: `deps(deps): bump the rust-ecosystem group with 4 updates`
✅ Correct: `chore(deps): bump the rust-ecosystem group with 4 updates`

### Other dependency examples:
- `build(deps): update rust toolchain to 1.70`
- `fix(deps): update serde to fix security vulnerability`
- `chore(deps): bump minor versions for compatibility`

### Code changes:
- `feat(auth): add user authentication system`
- `fix(api): resolve memory leak in endpoint handler`
- `refactor(utils): simplify error handling logic`
- `docs(readme): update installation instructions`
- `test(api): add integration tests for user endpoints`
- `ci(workflows): add automated deployment pipeline`
- `style(code): format code with rustfmt`
- `perf(cache): optimize database query performance`

## Breaking Changes:
Add `!` before the colon for breaking changes:
- `feat(api)!: remove deprecated endpoints`
- `refactor(core)!: change public API signature`

## Body and Footer:
Optional detailed description and footer:
```
feat: allow provided config object to extend other configs

BREAKING CHANGE: `extends` key in config file is now used for extending other config files
```

## Your Specific Fix:
For the commit: `167e42e deps(deps): bump the rust-ecosystem group with 4 updates`

Change it to: `chore(deps): bump the rust-ecosystem group with 4 updates`
