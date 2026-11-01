# Contributing to rcal

Thank you for your interest in contributing to **rcal**! We welcome contributions from everyone, whether it's filing bug reports, requesting features, improving documentation, or submitting code changes.

---

## 🛠️ Getting Started

### Prerequisites

* **Rust 1.80+** (install via [rustup.rs](https://rustup.rs))
* A terminal emulator with Unicode and mouse event support (e.g., Alacritty, Ghostty, Kitty, iTerm2, WezTerm, Windows Terminal).

### Setup & Local Development

```bash
# Clone the repository
git clone https://github.com/eeyees/rcal.git
cd rcal

# Check that everything builds
cargo check

# Run tests
cargo test

# Run rcal locally
cargo run
```

---

## 🏗️ Architecture Overview

Before making large changes, it helps to understand the core architecture:

* **`src/app.rs`**: Central state machine (`App` struct), managing views, navigation, selected dates, active modals, and HitMaps.
* **`src/events.rs`**: Crossterm event loop, handling keyboard shortcuts, transparent Korean IME mapping, and mouse interactions.
* **`src/hit_map.rs`**: Mouse event hit-testing engine mapping terminal cell coordinates `(x, y)` to interactive actions.
* **`src/ui/`**: Ratatui rendering components (`month_view`, `week_view`, `day_view`, `agenda_view`, `header`, `footer`, `sidebar`, `modals/`, and `theme`).
* **`src/storage/`**: Embedded SQLite database (`rusqlite`) running in WAL mode for sub-millisecond local queries.
* **`src/sync/`**: Asynchronous Tokio background worker syncing CalDAV (Apple iCloud, Naver Calendar), Google, MS 365, and local icalendar files.
* **`src/i18n/`**: Multi-language localization engine and transparent Vim Hangul mapping.

---

## 🧪 Testing Guidelines

We prioritize stability and responsiveness. All PRs should ensure tests pass:

```bash
# Run all unit and integration tests
cargo test

# Run tests with output printed
cargo test -- --nocapture
```

If you add a new feature or fix a bug, please include corresponding test coverage under the `tests/` directory.

---

## 📝 Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

* `feat(...)`: New feature (e.g., `feat(theme): add solarized palette`)
* `fix(...)`: Bug fix (e.g., `fix(modal): fix button hit-target bounds`)
* `docs(...)`: Documentation only changes
* `test(...)`: Adding or refactoring tests
* `refactor(...)`: Code change that neither fixes a bug nor adds a feature
* `perf(...)`: Performance improvement

---

## 🚀 Submitting a Pull Request

1. **Fork** the repository and create your branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. Make your changes and write unit/integration tests where applicable.
3. Verify that `cargo test` passes.
4. Commit your changes with a clear commit message.
5. Push to your fork and submit a **Pull Request** against the `main` branch.
