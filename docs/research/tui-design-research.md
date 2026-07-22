# TUI Design Research: State of the Art (2025-2026)

## 1. Best-in-Class TUI Applications

### Tier 1: Gold Standard Visual Design

**Lazygit** (Go/Bubbletea)
- Multi-panel layout with clear visual hierarchy through border colors
- Active panel highlighted with green/bold border; inactive panels use default/muted borders
- Contextual tab indicators within panels
- Status bar with context-sensitive keybinding hints
- Consistent panel-based navigation where the right panel responds to left-panel selection
- Design philosophy: always show all panels, make focus obvious, keep navigation predictable

**Yazi** (Rust)
- Three-column Miller column layout (parent / current / preview)
- Nerd Font icons as default for file type indicators
- Image/video preview directly in terminal (sixel/kitty protocol)
- Layered theming system: preset -> flavor -> user overrides
- Status bar adapts to narrow widths gracefully
- Unicode border symbols (`│`) with consistent styling

**Zellij** (Rust)
- Floating panes with drop shadows
- Tab bar with clear indicators
- Mode-based UI that transforms the footer bar based on current mode
- Beautiful color-coded pane management
- Sidebar reveal/open animations

**Helix** (Rust)
- Clean, minimal editor chrome
- Status line with mode indicator, file info, cursor position
- Tree-sitter powered syntax highlighting with semantic colors
- Inline diagnostic rendering
- Gutter with line numbers and git status indicators

**Bottom (btm)** (Rust/Ratatui)
- Real-time sparkline charts and graphs
- Color-coded resource usage with gradients
- Multiple layout modes with consistent styling
- Smooth data updates without flicker

### Tier 2: Notable Visual Design

**Atuin** - Full-screen search TUI with clean filtering UI
**GitUI** (Rust) - Fast git interface with staged/unstaged visual diff
**Spotify TUI / spotatui** - Media player interfaces with progress bars and album art
**Television** - Fuzzy finder with clean results layout
**Oatmeal** - Chat bubbles and slash commands for LLM interaction

### What Makes Them Beautiful

1. **Restrained color use** - Not everything is colorful; color has meaning
2. **Consistent spacing** - Panels breathe; content doesn't touch borders
3. **Clear focus indicators** - Always obvious which element is active
4. **Information layering** - Progressive disclosure, not everything at once
5. **Smooth updates** - No flicker, intelligent diffing, feels responsive
6. **Purposeful borders** - Borders create hierarchy, not decoration

---

## 2. Modern TUI Design Principles

### Typography and Spacing

**Breathing Room (Padding/Margins)**
- Content should NEVER touch borders directly. Always have at least 1 character of horizontal padding inside bordered panels
- Use `Block::new().padding(Padding::horizontal(1))` as minimum; `Padding::new(1, 1, 0, 0)` (left, right, top, bottom) for most content areas
- Between sections, use 1 empty line as visual separator
- Grid system: think in units of 1 character width/height, use consistent multiples

**Line Length and Density**
- Ideal line length: 40-80 characters for readability
- Don't fill the entire terminal width with text; leave margins
- For list items, left-align and keep consistent indentation
- Use truncation with ellipsis (`...`) rather than wrapping in tight spaces

**Vertical Rhythm**
- Group related items together with no spacing
- Separate groups with 1 blank line
- Section headers get 1 blank line above, 0 below
- Status bars and footers should have consistent height (1-2 lines)

### Color Usage and Palette Design

**The 60-30-10 Rule for TUIs**
- 60% - Base/background color (terminal default or theme base)
- 30% - Secondary colors (borders, muted text, surface colors)
- 10% - Accent colors (active elements, highlights, errors)

**Modern Palettes (Community Standards)**
- **Catppuccin** - The dominant modern theme. Four flavors: Latte (light), Frappe, Macchiato, Mocha (dark). Has first-class Rust/Ratatui integration via the `catppuccin` crate with `ratatui` feature flag
- **Gruvbox** - Warm retro aesthetic, excellent for readability
- **Dracula** - Vivid, high-contrast, good for dark terminals
- **Rose Pine** - Elegant, muted palette
- **Tokyo Night** - Modern, easy on the eyes

**Color Principles**
- Red = errors/destructive only. Never decorative red
- Green = success/active. Use for focused borders, confirmations
- Yellow/amber = warnings, caution states
- Blue = informational, links, primary actions
- Dim/gray = secondary text, disabled states, less important info
- Don't rely on color alone to convey meaning (accessibility)
- Support `NO_COLOR` environment variable
- Detect truecolor support; fall back to 256-color or 16-color gracefully

**Catppuccin Palette Structure for Ratatui**
```
Base colors:     base, mantle, crust (backgrounds, darkest to lightest)
Surface colors:  surface0, surface1, surface2 (elevated surfaces)
Overlay colors:  overlay0, overlay1, overlay2 (floating elements)
Text colors:     text, subtext0, subtext1 (content hierarchy)
Accent colors:   rosewater, flamingo, pink, mauve, red, maroon,
                 peach, yellow, green, teal, sky, sapphire, blue, lavender
```

### Border Styles and Visual Hierarchy

**Border Types (Best to Worst for Modern Look)**
1. **Rounded** (`╭───╮ │ │ ╰───╯`) - Best for modern, friendly appearance
2. **Plain/Single** (`┌───┐ │ │ └───┘`) - Classic, professional
3. **McGugan style** (half-block Unicode chars) - Tightest, most modern look using `▁▔▕▏` characters. Allows independent inside/outside colors
4. **Thick/Heavy** (`┏━━━┓ ┃ ┃ ┗━━━┛`) - For emphasis or primary containers
5. **Double** (`╔═══╗ ║ ║ ╚═══╝`) - Dated look; use sparingly or not at all

**Visual Hierarchy Through Borders**
- Active/focused panel: colored border (green or primary accent) + BOLD
- Inactive panels: dim/default border color
- Modal/popup: double or thick border to indicate overlay importance
- Nested sections: consider no border, just padding + background color change
- Collapsing borders: when panels are adjacent, share border lines (like CSS `border-collapse`)

**The McGugan Border Technique**
Will McGugan's innovation uses Unicode half-block and eighth-block characters:
- `U+2581` LOWER ONE EIGHTH BLOCK
- `U+2594` UPPER ONE EIGHTH BLOCK
- `U+258E` LEFT ONE QUARTER BLOCK
- `U+1FB87` RIGHT ONE QUARTER BLOCK (Unicode 13)
These allow borders where the inside and outside colors are independent, creating tighter, more precise panel boundaries than traditional box-drawing characters.

### Information Density vs Whitespace

**The Balance**
- Terminals have limited real estate; every character matters
- But cramming information makes it unreadable
- Goal: "scannable density" - lots of info that's easy to skim

**Techniques**
- Use dim/muted colors for secondary information rather than hiding it
- Align data in columns so the eye can scan vertically
- Use icons/symbols to replace words where unambiguous (e.g., `✓` vs "Success")
- Progressive disclosure: show summary, expand on focus/selection
- Truncate with `…` rather than wrapping
- Use sparklines and mini-charts for trends (1-line height)

**Information Hierarchy**
```
Most prominent:  Active/selected item, errors, current state
Medium:          Panel titles, section headers, primary data
Subtle:          Secondary data, timestamps, metadata
Minimal:         Borders, separators, background patterns
```

### Status Bars and Footer Design

**Standard Convention**
- Bottom 1-2 lines of terminal, full width
- Left side: mode indicator, current context (file name, branch, etc.)
- Center: optional status messages, progress
- Right side: position info, counts, timestamps

**Keybinding Hints**
- Format: `key:action` pairs separated by spaces or `│`
- Common style: `[q]uit  [enter]select  [/]search  [?]help`
- Or: `q Quit │ Enter Select │ / Search │ ? Help`
- Context-sensitive: change hints based on current mode/panel
- Use dim/muted color for the hint text, brighter for the key itself
- Cycle hints if too many to fit (rotate every 5-10 seconds)

**Design Patterns**
```
┌─ Mode indicator      ┌─ Context info         ┌─ Position/stats
│                       │                       │
NORMAL │ main.rs [+]                           42:12 │ UTF-8 │ LF
```

### Dialog/Modal Design

**Popup/Modal Conventions**
- Center the dialog both horizontally and vertically
- Use `Clear` widget to wipe the area behind the modal
- Add a visible border (thicker or different style than panels)
- Dim or blur the background content (tachyonfx can help)
- Size: typically 40-60% of terminal width, auto-height for content
- Always show how to dismiss: `[Esc] Cancel  [Enter] Confirm`
- Use `Padding::new(2, 2, 1, 1)` inside modals for breathing room

**Confirmation Dialogs**
```
╭──── Confirm Delete ────╮
│                         │
│  Delete 3 files?        │
│  This cannot be undone. │
│                         │
│   [Enter] Yes  [Esc] No │
╰─────────────────────────╯
```

### Scrolling and Scrollbar Design

**Scrollbar Conventions**
- Use thin scrollbar on the right edge of scrollable areas
- Track character: `│` or dim block
- Thumb character: `█` or `┃` (make it stand out from track)
- Position scrollbar inside the border using `area.inner(Margin { vertical: 1, horizontal: 0 })`
- Only show scrollbar when content overflows (hide when everything fits)
- Ratatui's built-in `Scrollbar` widget supports `.thumb_symbol()`, `.track_symbol()`, `.begin_symbol()`, `.end_symbol()`

**Scroll Indicators**
- Top/bottom arrows: `▲` / `▼` to indicate more content
- Or use fade-out effect on first/last visible lines
- Show scroll position as percentage or `3/42` style counter

### Animation and Transitions

**TachyonFX (Ratatui's Animation Library)**
- Operates on terminal cells AFTER widgets render
- Effects are stateful: create once, apply every frame
- Key effects available:
  - `slide_in` / `slide_out` - directional panel transitions
  - `dissolve` / `coalesce` - text materialization/dissolution
  - `fade_in` / `fade_out` - opacity transitions via color interpolation
  - Radial patterns expanding from center
  - `CellFilter` for targeting specific cells
- Compose effects: parallel and sequential effect chains
- Use sparingly: animations should aid comprehension, not distract
- Good uses: modal appear/disappear, tab transitions, notification toasts

**Performance Target**
- Ratatui achieves 60+ FPS with complex layouts via intelligent buffer diffing
- Animations should complete in 100-300ms for snappy feel
- Never block the main loop for animation

### Icon Usage

**Nerd Font Icons**
- Prerequisite: user must have a Nerd Font installed
- Common icons for TUI apps:
  - File types: `  ` (folder, file, code)
  - Git: `    ` (branch, commit, merge, compare)
  - Status: ` ✓  ` (error, success, warning, info)
  - Navigation: `    ` (arrows, chevrons)
  - Actions: `    ` (search, edit, delete, copy)
- Provide graceful fallback for terminals without Nerd Fonts (ASCII alternatives)

**Unicode Symbols (No Nerd Font Required)**
- Bullets: `●` `○` `◆` `◇`
- Arrows: `→` `←` `↑` `↓` `▶` `◀`
- Status: `✓` `✗` `⚠` `ℹ`
- Progress: `█` `▓` `░` `▒`
- Separators: `│` `─` `·` `•`
- Spinners: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` (braille animation frames)

---

## 3. Ratatui-Specific Patterns

### Architecture

**The Elm Architecture (TEA)** - Recommended pattern:
```
Model (state) -> View (render) -> Update (handle events) -> Model
```

**Component Pattern** - For complex UIs:
- Each panel/section is a component with its own state, render, and update
- Parent component manages layout and inter-component communication

### Essential Crates for Beautiful UIs

| Crate | Purpose |
|-------|---------|
| `ratatui` | Core TUI framework |
| `crossterm` | Terminal backend |
| `catppuccin` (with `ratatui` feature) | Color palette integration |
| `tachyonfx` | Animation and shader-like effects |
| `ratatui-image` | Image rendering (sixel/halfblock) |
| `tui-scrollview` | Scrollable viewports |
| `tui-popup` | Popup/modal overlays |
| `ratatui-garnish` | Composable widget decorators (borders, shadows, padding) |
| `ratatui-code-editor` | Syntax-highlighted code editing |

### Block and Layout Patterns

**Standard Panel**
```rust
let block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(if focused {
        Style::default().fg(theme.primary).bold()
    } else {
        Style::default().fg(theme.surface2)
    })
    .title(title)
    .title_style(Style::default().fg(theme.text).bold())
    .padding(Padding::horizontal(1));
```

**Layout with Consistent Gaps**
```rust
let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(30),
        Constraint::Percentage(70),
    ])
    .split(area);
```

**Centered Modal**
```rust
let popup_area = area.inner(Margin { vertical: area.height / 4, horizontal: area.width / 4 });
frame.render_widget(Clear, popup_area);
frame.render_widget(popup_block, popup_area);
```

### Color Integration Pattern

```rust
use catppuccin::Flavour;

struct Theme {
    base: Color,
    surface0: Color,
    surface1: Color,
    text: Color,
    subtext0: Color,
    primary: Color,     // e.g., blue or sapphire
    secondary: Color,   // e.g., teal
    success: Color,     // green
    warning: Color,     // yellow
    error: Color,       // red
    accent: Color,      // mauve or lavender
}

impl From<Flavour> for Theme {
    fn from(flavour: Flavour) -> Self {
        let colors = flavour.colours();
        Theme {
            base: Color::Rgb(colors.base.0, colors.base.1, colors.base.2),
            // ... map all colors
        }
    }
}
```

### Showcase Applications Built with Ratatui

- **bottom (btm)** - System monitor with charts and sparklines
- **gitui** - Git interface with diff rendering
- **Television** - Fuzzy finder
- **spotatui** - Spotify client with visualizations
- **Oatmeal** - LLM chat interface with bubbles
- **Maze TUI** - Algorithm visualization
- **b-top** - Process monitor using tachyonfx for shader effects

---

## 4. Common Anti-Patterns (What Makes TUIs Look Bad)

### Visual Anti-Patterns

1. **Color Vomit** - Using too many colors with no system. Every element a different color. As CLI Guidelines states: "if everything is a different color, then the color means nothing"

2. **Border Overload** - Putting borders on everything. Nested borders within borders. Creates visual noise. Instead: use spacing and background color changes for inner sections

3. **No Focus Indicator** - When it's unclear which panel/element is active. All borders look the same. User doesn't know where they are

4. **Wall of Text** - No visual hierarchy. Everything at the same font weight and color. No grouping or sectioning. No icons or symbols to break monotony

5. **Cramped Content** - Text touching borders. No padding anywhere. Everything packed tight with no breathing room

6. **Double Borders** - Using `╔═══╗` style everywhere. Looks dated and heavy. Rounded borders (`╭───╮`) are the modern standard

7. **Inconsistent Alignment** - Mix of left, center, right aligned text with no system. Columns that don't line up. Ragged data presentation

8. **Raw Data Dump** - Showing unformatted data without structure. No column alignment. No truncation. Long strings wrapping awkwardly

9. **Flicker and Jank** - Not using differential rendering. Clearing and redrawing the whole screen every frame. Visible flash when switching states

10. **Mysterious Shortcuts** - No visible keybinding hints. User has to memorize or guess. No status bar with current available actions

### Behavioral Anti-Patterns

11. **Silent Long Operations** - No progress indicator. User wonders if the program is frozen. Always show spinners, progress bars, or at minimum a "working..." message

12. **Modal Traps** - Modals with no obvious way to dismiss. Missing `[Esc]` hint. User feels stuck

13. **Overuse of Animation** - Animations that delay user interaction. Effects that play on every keystroke. Transitions that take more than 300ms

14. **Breaking Terminal Conventions** - Reinventing navigation keys. Not supporting standard keybindings (q to quit, / to search, ? for help). Users expect Vim-like or Emacs-like navigation

15. **No Responsive Layout** - Fixed-width layouts that break on small terminals. No minimum size handling. Content that overlaps when terminal is resized

---

## 5. Design Inspirations: Detailed Analysis

### Lazygit
**What to steal:**
- Panel-based layout where all panels are always visible
- Active border = green + bold, inactive = dim
- Tab indicators within panel headers (`1 Local Branches │ 2 Remotes │ 3 Tags`)
- Context-sensitive footer with keybinding hints
- Confirmation dialogs that are centered, concise, and show keys
- Diff view with syntax-colored additions/deletions
- Search mode that highlights matches across panels

**Key technique:** Strong consistency. Every panel behaves the same way. Navigation is predictable. The UI never surprises you.

### Yazi
**What to steal:**
- Miller column three-pane layout for hierarchical data
- Nerd Font icons for every file type (with ASCII fallback)
- Preview pane that adapts content type (text, image, directory listing)
- Minimal chrome - very little border decoration, lets content breathe
- Status bar that gracefully degrades in narrow terminals
- Smooth scrolling within columns
- Theme/flavor system allowing deep customization

**Key technique:** Content-first design. The UI gets out of the way. Maximum screen real estate for actual data.

### Zellij
**What to steal:**
- Mode-based footer that transforms completely per mode
- Floating panes with shadow/depth effect
- Tab bar with indicators and easy switching
- Color-coded modes (normal = green, pane = blue, resize = yellow)
- Clean separator lines between panes
- Compact mode that hides chrome for more content space

**Key technique:** Mode-awareness. The UI itself communicates what mode you're in through color changes in the chrome, not just content.

### Helix
**What to steal:**
- Minimal editor chrome with maximum content area
- Status line: mode | file | diagnostics | position | encoding
- Gutter design: line numbers + git indicators in same column
- Inline diagnostics with virtual text
- Command palette / picker with fuzzy search
- Muted line numbers that don't compete with content
- Selection highlighting that's visible but not overwhelming

**Key technique:** Typographic hierarchy. Different weights and colors for different levels of importance. The code itself is the star.

### Atuin
**What to steal:**
- Full-screen search overlay that appears on `Ctrl+R`
- Clean list of results with metadata (duration, exit code, date)
- Inline filtering that updates results in real-time
- Keyboard-first design with clear navigation
- Minimal decoration; information speaks for itself

**Key technique:** Single-purpose excellence. Does one thing (search history) with maximum visual clarity.

---

## Summary: The Formula for a Beautiful TUI

1. **Pick a real color palette** (Catppuccin Mocha recommended for dark themes)
2. **Use rounded borders** for a modern feel
3. **Active = colored + bold border**, inactive = dim border
4. **Always pad content** 1 char inside borders minimum
5. **Status bar at bottom** with context-sensitive keybinding hints
6. **Progressive disclosure** - don't show everything at once
7. **Icons (Nerd Font)** for type indicators, with Unicode/ASCII fallback
8. **Dim secondary info** rather than hiding it
9. **Center modals** with clear dismiss hints
10. **Animate sparingly** - 100-300ms transitions for state changes
11. **Scrollbars only when needed**, thin and subtle
12. **Consistent spacing** - pick a system and stick with it
13. **Test at multiple terminal sizes** - responsive layout matters
14. **Support NO_COLOR** and graceful color degradation
