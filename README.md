# rcal (Rust TUI Calendar)

> **Fast, Modern, Mouse-interactive Terminal Calendar in Rust with Multi-Cloud Sync**

`rcal` is a lightweight, responsive TUI calendar built for developers and terminal enthusiasts who want the visual power of Google Calendar directly in their terminal with zero lag.

---

## ✨ Features (Step 1 MVP Completed)

- ⚡ **Zero-lag Startup**: Instant offline-first load using an embedded SQLite database (`rusqlite` WAL mode).
- 🖱️ **Full Mouse Interactivity**:
  - Click view tabs to switch views (`[ 1:Month ]`, `[ 2:Week ]`, `[ 3:Day ]`, `[ 4:Agenda ]`).
  - Click `[ < ]`, `[ Today ]`, `[ > ]` to navigate through time.
  - Click any day cell in Month view to select date; double-click to drill down into Day view.
  - Click any time slot in Week or Day view to create an event at that specific hour.
  - Click any event block/snippet to open its details modal.
  - Click sidebar calendar checkboxes (`[x] Personal`, `[x] Work`) to toggle visibility.
  - Mouse wheel scrolling to navigate through dates.
- 📅 **4 Familiar Views**:
  - **Month View**: 7-column calendar with event indicators, today badges, and date highlights.
  - **Week View**: Sunday–Saturday timeline with hour slots (08:00–22:00) and colored event blocks.
  - **Day View**: Single-day schedule with side-by-side display for concurrent/overlapping events.
  - **Agenda View**: Chronological upcoming event stream with date grouping.
- 📝 **Event CRUD Modal**:
  - Create, view, edit, and delete events with date/time, location, and description.
  - Color-coded calendars (Personal, Work, Holidays).
- ⌨️ **Vim & Intuitive Keybindings**:
  - `h/j/k/l`: Move cursor (Left/Down/Up/Right).
  - `1`, `2`, `3`, `4` or `m`, `w`, `d`, `a`: View switching.
  - `c` or `n`: Create new event.
  - `t`: Jump to today.
  - `[` / `]`: Previous / Next period.
  - `?`: Toggle help screen.
  - `q` or `Esc`: Close modal or quit.

---

## 🛠️ Tech Stack

- **UI & Terminal**: [Ratatui](https://github.com/ratatui/ratatui) + [Crossterm](https://github.com/crossterm-rs/crossterm)
- **Local Storage**: SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled)
- **Time & Scheduling**: [chrono](https://github.com/chronotope/chrono), `chrono-tz`
- **Async Runtime**: [tokio](https://tokio.rs)

---

## 🚀 Getting Started

### Prerequisites

- Rust 1.80+ (`rustup`)

### Build & Run

```bash
# Clone the repository (or navigate to workspace)
cd rcal

# Run debug build
cargo run

# Build optimized release binary
cargo build --release
./target/release/rcal
```

### Running Tests

```bash
cargo test
```

---

## 🗺️ Roadmap

- [x] **Step 1**: Local TUI MVP with Month/Week/Day/Agenda views, Mouse hit-testing, SQLite persistence, and Event CRUD.
- [x] **Step 2**: CalDAV Sync Engine (Apple iCloud & Naver Calendar two-way synchronization, Background Tokio worker, Account modal).
- [ ] **Step 3**: OAuth2 Cloud Adapters (Google Calendar API & Microsoft 365 Graph API).
- [ ] **Step 4**: Recurrence rules (RRULE), Quick natural-language event add, and OS notifications.
