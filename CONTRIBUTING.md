# Contributing to HiNotes Desktop

Thank you for your interest in contributing to HiNotes Desktop!

## Test-Driven Development (TDD)

This project follows strict TDD methodology. Every feature must follow the Red-Green-Refactor cycle:

1. **RED**: Write a failing test
2. **GREEN**: Write minimal code to make the test pass
3. **REFACTOR**: Improve code quality while keeping tests green

### Running Tests

```bash
# Rust backend tests
cargo test

# Frontend tests
npm test

# E2E tests
npx playwright test
```

## Commit Guidelines

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Adding tests
- `refactor`: Code refactoring
- `chore`: Build/tooling

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
