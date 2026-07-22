# TUI Visual Polish - Phased Implementation Plan

## Context

Conduit's TUI has excellent infrastructure (50+ semantic color tokens, lock-free theme system, VS Code import, WCAG contrast enforcement) but the rendered output doesn't yet leverage it fully. A visual audit identified spacing, message differentiation, and input area polish as the three highest-impact, lowest-effort improvements.

---

## Phase 1: High-Impact Visual Polish (Top 3 Items) ✅ DONE

### 1.1 Content Breathing Room ✅
- Chat content left margin increased from 2 to 4 columns
- Input area margins increased from 2 to 4 columns

### 1.2 User vs. Assistant Message Differentiation ✅
- Added `agent_label` field to ChatView (set from AgentType::short_name())
- "You" label above user messages
- Agent name label (e.g. "Claude") above assistant messages

### 1.3 Input Area Placeholder ✅
- Placeholder "Type a message..." shown when input is empty (text_faint color)
- Note: Separator line was implemented then reverted per user preference

---

## Phase 2: Status Bar & Footer Cleanup

### 2.1 Merge status bar + footer chrome
- Combine the `▀▀▀` gap row and two information strips into cleaner treatment
- Replace heavy `▀` block separator with a thin `─` line or nothing
- Left: model/agent/mode, right: branch/PR, center: keybindings

### 2.2 Tab bar active indicator
- Add underline (`━` or `▁`) below active tab
- Improve inactive tab contrast

### 2.3 Summary divider styling
- Color arrows: `↓` in `accent_primary`, `↑` in `accent_secondary`
- Use `text_faint` for `─` fill, `text_muted` for metrics text
- Center metrics in the divider

---

## Phase 3: Dialog & Sidebar Polish

### 3.1 Dialog shadows (1-cell `░` on right + bottom edges)
### 3.2 Sidebar tree connectors (`├──`, `└──`, `│`)
### 3.3 Welcome/empty state (centered logo + hints)

---

## Phase 4: Micro-Animations

### 4.1 Evaluate `tachyonfx` for dialog fade, tab highlight, smooth scroll

---

## Verification

After each phase:

1. `cargo build --release` - clean compile
2. `cargo test` - all pass
3. Visual verification at 120x40 and 80x24
4. Theme check: Switch between default-dark, catppuccin-mocha, tokyo-night
