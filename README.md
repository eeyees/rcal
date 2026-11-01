<div align="center">

```
  ██████╗  ██████╗ █████╗ ██╗     
  ██╔══██╗██╔════╝██╔══██╗██║     
  ██████╔╝██║     ███████║██║     
  ██╔══██╗██║     ██╔══██║██║     
  ██║  ██║╚██████╗██║  ██║███████╗
  ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝
```

### **Fast, Modern, Mouse-interactive Terminal Calendar in Rust with Multi-Cloud Sync**

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg?style=flat-square)](https://github.com/eeyees/rcal)
[![Ratatui](https://img.shields.io/badge/Ratatui-0.29-green.svg?style=flat-square&logo=terminal)](https://github.com/ratatui/ratatui)
[![SQLite](https://img.shields.io/badge/SQLite-WAL%20Mode-003B57.svg?style=flat-square&logo=sqlite)](https://www.sqlite.org)
[![Tokio](https://img.shields.io/badge/Async-Tokio-red.svg?style=flat-square)](https://tokio.rs)

<p align="center">
  <a href="#-key-features">Key Features</a> •
  <a href="#-ui-views--preview">UI Views</a> •
  <a href="#-keyboard--mouse-controls">Controls</a> •
  <a href="#-multi-cloud--caldav-sync">Cloud Sync</a> •
  <a href="#-themes--customization">Themes</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-architecture">Architecture</a>
</p>

---

</div>

`rcal` is a lightweight, responsive TUI calendar designed for developers, terminal enthusiasts, and power users who want the visual clarity and ease-of-use of Google Calendar directly in their terminal with **zero lag**, **100% mouse interactivity**, and **seamless multi-cloud synchronization**.

---

## ✨ Key Features

- ⚡ **Zero-Lag Offline-First Architecture**: Instant sub-millisecond startup backed by an embedded SQLite database running in WAL (*Write-Ahead Logging*) mode.
- 🖱️ **Full Mouse Interactivity**:
  - Click view tabs (`[ 1:Month ]`, `[ 2:Week ]`, `[ 3:Day ]`, `[ 4:Agenda ]`) or header buttons (`[ < ]`, `[ Today ]`, `[ > ]`, `[ ⟳ Sync ]`).
  - Click any day cell in Month view to select; double-click to drill down into Day view.
  - Click any hour slot in Week or Day view to quickly draft an event.
  - Click event blocks to open interactive details and edit modal.
  - Mouse wheel scrolling across days and calendar timelines.
- 📅 **4 Familiar, High-Density Views**:
  - **Month View**: 7-column calendar layout with event badges, today highlights, and date indicators.
  - **Week View**: Sunday–Saturday (or Monday/Saturday start) timeline with hourly slots and colored event rectangles.
  - **Day View**: High-precision single-day schedule with intelligent side-by-side positioning for concurrent/overlapping events.
  - **Agenda View**: Chronological upcoming event stream grouped by date.
- ☁️ **Multi-Cloud & CalDAV Sync Engine**:
  - **Apple iCloud**: Native two-way CalDAV synchronization.
  - **Naver Calendar**: CalDAV sync integration with auto-discovery.
  - **Google Calendar**: Google API OAuth2 adapter.
  - **Microsoft 365 / Outlook**: Microsoft Graph API OAuth2 engine.
  - **Samsung Calendar**: Cloud adapter integration.
  - **Background Worker**: Dedicated Tokio background task syncing silently every 5 minutes or on-demand without freezing the UI.
- ⌨️ **Vim Navigation & Transparent Korean IME Handling**:
  - Classic `h` / `j` / `k` / `l` navigation, tab switching, and modal controls.
  - **Transparent IME Translation**: Use Vim shortcuts seamlessly even when typing with Korean (`ㅗ/ㅓ/ㅏ/ㅣ`) input mode active without annoying terminal beeps or missed keystrokes.
- 🌐 **Native Multi-Language (i18n) Support**:
  - Automatic system locale detection (`LANG` / `LC_ALL`).
  - Runtime 1-key language toggle between **한국어 (Korean)** and **English** with `L` or mouse click.
  - Localized dates, day names, modals, forms, and status notifications.
- 🎨 **Rich Themes & Customization**:
  - 7 Built-in color palettes: **Modern Dark**, **Nord**, **Dracula**, **Forest**, **Sunset**, **Monokai**, and **Solarized**.
  - Configurable week start day: **Sunday**, **Monday**, or **Saturday**.
  - Custom calendar creation with color tags (`Personal`, `Work`, `Holidays`, etc.).

---

## 🖥️ UI Views & Preview

```
┌── rcal v0.1.0 ─────────────────────────────────────────────── [ L: 한/EN ] [ ? Help ] [ ⚙ Settings ] ──┐
│ [ 1:Month ] [ 2:Week ] [ 3:Day ] [ 4:Agenda ]    [ < ] [ Today ] [ > ]  2026-11-01 (Sun)   [ ⟳ Sync ]  │
├────────────────────┬───────────────────────────────────────────────────────────────────────────────────┤
│ Calendars          │                             November 2026                                         │
│ [x] ■ Personal     │  Sun         Mon         Tue         Wed         Thu         Fri         Sat      │
│ [x] ■ Work         │ ┌───────────┬───────────┬───────────┬───────────┬───────────┬───────────┬───────────┐ │
│ [x] ■ Holidays     │ │ 1  TODAY  │ 2         │ 3         │ 4         │ 5         │ 6         │ 7         │ │
│                    │ │■ 10:00 AM │■ 09:00 AM │           │■ 14:00 PM │           │■ 19:00 PM │           │ │
│ Accounts           │ │  Sprint   │  Standup  │           │  Design   │           │  Dinner   │           │ │
│ • iCloud (Syncing) │ ├───────────┼───────────┼───────────┼───────────┼───────────┼───────────┼───────────┤ │
│ • Google (Active)  │ │ 8         │ 9         │ 10        │ 11        │ 12        │ 13        │ 14        │ │
│                    │ │           │           │           │           │           │           │           │ │
├────────────────────┴───────────────────────────────────────────────────────────────────────────────────┤
│ [c] New Event  [m/w/d/a] Switch View  [h/j/k/l] Navigate  [/] Search  [a] Accounts  [s] Settings  [q] Quit │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## ⌨️ Keyboard & Mouse Controls

### 🧭 Navigation & Views

| Keybinding | Mouse Action | Description |
| :--- | :--- | :--- |
| `h` / `j` / `k` / `l` or Arrows | Click date cell / time slot | Move focus Left / Down / Up / Right |
| `1` or `m` | Click `[ 1:Month ]` tab | Switch to **Month View** |
| `2` or `w` | Click `[ 2:Week ]` tab | Switch to **Week View** |
| `3` or `d` | Click `[ 3:Day ]` tab | Switch to **Day View** |
| `4` or `a` | Click `[ 4:Agenda ]` tab | Switch to **Agenda View** |
| `t` | Click `[ Today ]` button | Jump cursor to Today |
| `[` / `]` | Click `[ < ]` / `[ > ]` buttons | Previous / Next period (Month / Week / Day) |
| Scroll Up / Down | Mouse Wheel | Scroll timeline or cycle through dates |
| Double-Click | Double-Click on day cell | Drill down directly from Month view into Day view |

### 📝 Events & Modal Management

| Keybinding | Mouse Action | Description |
| :--- | :--- | :--- |
| `c` or `n` | Click `[ + New ]` / empty slot | Create new event form modal |
| `Enter` | Click on event box | Open selected event details modal |
| `e` | Click `[ Edit ]` button | Edit event |
| `x` or `Delete` | Click `[ Delete ]` button | Delete selected event |
| `Tab` / `Shift+Tab` | Click form field / button | Cycle between input fields and buttons |
| `Esc` or `q` | Click `[ Close ]` / outside backdrop | Close active modal |

### ⚙️ System & Customization

| Keybinding | Mouse Action | Description |
| :--- | :--- | :--- |
| `T` | Click theme in `[ ⚙ Settings ]` | Cycle color theme (Modern Dark, Nord, Dracula, Forest, Sunset, Monokai, Solarized) |
| `L` | Click `[ L: 한/EN/日 ]` | Cycle language (Korean ↔ English ↔ Japanese) |
| `s` | Click `[ ⚙ Settings ]` | Open Settings Modal (Week start day, themes, language) |
| `a` | Click Accounts in sidebar | Open Cloud Accounts Manager Modal |
| `b` | Click sidebar toggle | Toggle sidebar visibility |
| `r` | Click `[ ⟳ Sync ]` | Trigger immediate background cloud sync |
| `?` | Click `[ ? Help ]` | Toggle keyboard shortcuts cheat sheet |
| `Ctrl+C` / `q` | - | Quit `rcal` safely |

---

## ☁️ Multi-Cloud & CalDAV Sync

`rcal` includes built-in sync adapters designed to keep your terminal schedule in sync with all your devices.

```mermaid
flowchart LR
    A["rcal TUI"] <--> B["Local SQLite (WAL)"]
    B <--> C["Background Tokio Sync Worker"]
    C <--> D["Apple iCloud (CalDAV)"]
    C <--> E["Google Calendar (OAuth2)"]
    C <--> F["Microsoft 365 (Graph API)"]
    C <--> G["Naver Calendar (CalDAV)"]
    C <--> H["Samsung Calendar"]
```

### Supported Providers

- **Apple iCloud**: Connect with your Apple ID and an App-Specific Password via CalDAV.
- **Naver Calendar**: Connect using Naver CalDAV (`https://caldav.calendar.naver.com:443/caldav/`) and 2-step verification app password.
- **Google Calendar**: Secure token exchange via Google Cloud OAuth2 credentials.
- **Microsoft 365 / Outlook**: Enterprise and personal Outlook calendar sync via Microsoft Graph REST endpoints.
- **Samsung Calendar**: Samsung Cloud account integration.

To configure your accounts, press `a` inside `rcal` or select the **Accounts** menu in the sidebar.

---

## 🎨 Themes & Customization

Cycle themes quickly anytime by pressing **`T`** (`Shift+T`), or press **`s`** to open the **Settings Modal** to click and select:

### Built-in Color Palettes
- 🌙 **Modern Dark** (Default clean terminal look)
- ❄️ **Nord** (Arctic, north-bluish palette)
- 🧛 **Dracula** (High-contrast dark theme)
- 🌲 **Forest** (Natural matcha & earthy green tones)
- 🌆 **Sunset** (Vibrant cyberpunk purple & orange)
- 🎨 **Monokai** (Classic code editor contrast)
- ☀️ **Solarized** (Precision tuned low-contrast dark)

Selected themes are immediately persisted to the SQLite database and restored on startup.

### Week Start Preference
Choose whether your week starts on **Sunday**, **Monday**, or **Saturday** based on your cultural or regional workflow.

---

## 🏗️ Architecture

```mermaid
flowchart TD
    subgraph UI ["Terminal Presentation (Ratatui + Crossterm)"]
        EventLoop["Crossterm Event Stream (Key, Mouse, Resize)"]
        HitMap["HitMap & Mouse Engine"]
        Renderer["Ratatui Terminal Renderer"]
        Modals["Modal Stack (EventForm, Accounts, Settings, Help)"]
    end

    subgraph Core ["Core Engine"]
        AppState["App State Machine"]
        i18n["i18n Engine & Hangul Transparent IME"]
        ThemeEngine["Theme & Styling Engine"]
    end

    subgraph Storage ["Persistent Storage"]
        DB["SQLite Database (rusqlite WAL)"]
        Models["Models: Event, Calendar, Account"]
    end

    subgraph BackgroundWorker ["Asynchronous Network Layer"]
        Tokio["Tokio Background Task (300s ticker + mpsc trigger)"]
        SyncEngine["Sync Engine"]
        Adapters["Adapters (CalDAV, icalendar, Google, MS Graph)"]
    end

    EventLoop --> HitMap --> AppState
    AppState --> Renderer
    AppState --> Modals
    AppState <--> DB
    AppState --> Tokio
    Tokio <--> SyncEngine
    SyncEngine <--> DB
    SyncEngine <--> Adapters
```

---

## 🚀 Installation

### Prerequisites

- [Rust 1.80+](https://www.rust-lang.org/tools/install) (`cargo` & `rustup`)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/eeyees/rcal.git
cd rcal

# Build optimized release binary
cargo build --release

# Run rcal
./target/release/rcal
```

### Install Globally via Cargo

```bash
cargo install --path .
rcal
```

### Data Storage Location

- **macOS**: `~/Library/Application Support/rcal/rcal.db`
- **Linux**: `~/.local/share/rcal/rcal.db`
- **Windows**: `%LOCALAPPDATA%\rcal\rcal.db`

---

## 🗺️ Development Roadmap

- [x] **Phase 1: Local TUI MVP**
  - Month, Week, Day (with overlap detection), and Agenda views
  - 100% Mouse hit-testing engine
  - Embedded SQLite WAL database persistence
  - Event CRUD modal with full validation
- [x] **Phase 2: CalDAV Synchronization**
  - Apple iCloud two-way sync
  - Naver Calendar two-way sync
  - Background Tokio sync worker & auto-interval ticker
  - Account configuration UI
- [x] **Phase 3: Multi-Cloud Adapters & Polishing**
  - Google Calendar OAuth2 engine
  - Microsoft 365 Graph API adapter
  - Samsung Calendar adapter
  - Transparent Korean IME navigation mapping
  - Multiple color themes and week start day settings
- [ ] **Phase 4: Ecosystem & Extensibility**
  - [ ] Model Context Protocol (MCP) server support for AI agents
  - [ ] WebCal / iCalendar (`.ics`) local HTTP subscription server
  - [ ] Recurrence rules (RFC 5545 RRULE support)
  - [ ] Natural language event entry (e.g., `rcal add "Lunch tomorrow at 12pm"`)
  - [ ] Native OS notifications for upcoming events

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!  
Feel free to check out the [issues page](https://github.com/eeyees/rcal/issues).

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feat/AmazingFeature`)
3. Commit your Changes (`git commit -m 'feat: Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feat/AmazingFeature`)
5. Open a Pull Request

---

## 📜 License

Distributed under the **MIT License**. See `LICENSE` for more information.
