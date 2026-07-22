//! Add repository dialog component

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::path::PathBuf;

use super::{
    accent_primary, expand_tilde, DialogFrame, PathInputState, StatusLine, TextInputState,
};

/// Which tab of the dialog is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddRepoMode {
    /// Add a project that already exists on disk
    #[default]
    AddExisting,
    /// Create a brand-new project (folder + git init + first commit)
    CreateNew,
}

/// Which field of the "Create new project" tab has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewProjectField {
    /// Project name (becomes the folder name)
    #[default]
    Name,
    /// Folder the project directory is created in
    ParentFolder,
}

/// State for the add repository dialog
#[derive(Debug, Clone)]
pub struct AddRepoDialogState {
    /// Shared path input state (includes visibility and validation)
    pub path: PathInputState,
    /// Extracted repository name
    pub repo_name: Option<String>,
    /// Active tab
    pub mode: AddRepoMode,
    /// "Create new project": project name input
    pub new_name: TextInputState,
    /// "Create new project": parent folder input
    pub new_parent: TextInputState,
    /// "Create new project": focused field
    pub new_focus: NewProjectField,
    /// "Create new project": validation error, if any
    pub new_error: Option<String>,
    /// "Create new project": validated target path (set only when valid)
    pub new_project_path: Option<PathBuf>,
}

impl Default for AddRepoDialogState {
    fn default() -> Self {
        Self::new()
    }
}

impl AddRepoDialogState {
    pub fn new() -> Self {
        Self {
            path: PathInputState::new(),
            repo_name: None,
            mode: AddRepoMode::default(),
            new_name: TextInputState::new(),
            new_parent: TextInputState::new(),
            new_focus: NewProjectField::default(),
            new_error: None,
            new_project_path: None,
        }
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.path.show();
        self.path.text.clear();
        self.repo_name = None;
        self.mode = AddRepoMode::default();
        self.new_name.clear();
        self.new_parent.clear();
        self.new_focus = NewProjectField::default();
        self.new_error = None;
        self.new_project_path = None;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.path.hide();
    }

    /// Get the current input value
    pub fn input(&self) -> &str {
        self.path.input()
    }

    /// Switch to the other tab
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            AddRepoMode::AddExisting => AddRepoMode::CreateNew,
            AddRepoMode::CreateNew => AddRepoMode::AddExisting,
        };
    }

    /// Move focus to the next field ("Create new project" tab only)
    pub fn focus_next_field(&mut self) {
        if self.mode == AddRepoMode::CreateNew {
            self.new_focus = match self.new_focus {
                NewProjectField::Name => NewProjectField::ParentFolder,
                NewProjectField::ParentFolder => NewProjectField::Name,
            };
        }
    }

    /// Move focus to the previous field ("Create new project" tab only)
    pub fn focus_prev_field(&mut self) {
        // With two fields, previous and next are the same move.
        self.focus_next_field();
    }

    /// The text input the keyboard currently edits, if the new-project tab is active
    fn focused_new_input(&mut self) -> &mut TextInputState {
        match self.new_focus {
            NewProjectField::Name => &mut self.new_name,
            NewProjectField::ParentFolder => &mut self.new_parent,
        }
    }

    // Delegate text input methods with validation
    pub fn insert_char(&mut self, c: char) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.insert_char(c),
            AddRepoMode::CreateNew => self.focused_new_input().insert_char(c),
        }
        self.validate();
    }

    pub fn delete_char(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.delete_char(),
            AddRepoMode::CreateNew => self.focused_new_input().delete_char(),
        }
        self.validate();
    }

    pub fn delete_forward(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.delete_forward(),
            AddRepoMode::CreateNew => self.focused_new_input().delete_forward(),
        }
        self.validate();
    }

    pub fn move_left(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.move_left(),
            AddRepoMode::CreateNew => self.focused_new_input().move_left(),
        }
    }

    pub fn move_right(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.move_right(),
            AddRepoMode::CreateNew => self.focused_new_input().move_right(),
        }
    }

    pub fn move_start(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.move_start(),
            AddRepoMode::CreateNew => self.focused_new_input().move_start(),
        }
    }

    pub fn move_end(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.path.move_end(),
            AddRepoMode::CreateNew => self.focused_new_input().move_end(),
        }
    }

    /// Validate the active tab's input
    pub fn validate(&mut self) {
        match self.mode {
            AddRepoMode::AddExisting => self.validate_existing(),
            AddRepoMode::CreateNew => self.validate_new(),
        }
    }

    /// Trimmed project name for the "Create new project" tab
    pub fn new_project_name(&self) -> &str {
        self.new_name.value().trim()
    }

    /// Expanded parent folder for the "Create new project" tab
    pub fn new_parent_path(&self) -> PathBuf {
        expand_tilde(self.new_parent.value().trim())
    }

    /// Path the new project would be created at, as typed (for the preview line)
    pub fn new_project_preview(&self) -> Option<String> {
        let name = self.new_project_name();
        let parent = self.new_parent.value().trim();
        if name.is_empty() || parent.is_empty() {
            return None;
        }
        Some(format!("{}/{}", parent.trim_end_matches('/'), name))
    }

    /// Validate the "Create new project" inputs
    fn validate_new(&mut self) {
        let name = self.new_project_name().to_string();
        let parent_input = self.new_parent.value().trim().to_string();

        // Nothing typed yet: no error, just not valid.
        if name.is_empty() || parent_input.is_empty() {
            self.new_error = None;
            self.new_project_path = None;
            return;
        }

        // Same rules the creation itself enforces, so a valid-looking form
        // cannot fail on the checks at Enter time.
        match crate::git::validate_new_project_path(&expand_tilde(&parent_input), &name) {
            Ok(path) => {
                self.new_project_path = Some(path);
                self.new_error = None;
            }
            Err(e) => {
                self.new_project_path = None;
                self.new_error = Some(e.to_string());
            }
        }
    }

    /// Validate the existing-repository path
    fn validate_existing(&mut self) {
        let input = self.path.input();

        // Check if path is empty
        if input.is_empty() {
            self.path.set_invalid();
            self.repo_name = None;
            return;
        }

        // Expand ~ to home directory
        let expanded_path = self.path.expanded_path();

        // Check if path exists
        if !expanded_path.exists() {
            self.path.set_error("Path does not exist");
            self.repo_name = None;
            return;
        }

        // Check if it's a directory
        if !expanded_path.is_dir() {
            self.path.set_error("Path is not a directory");
            self.repo_name = None;
            return;
        }

        // Check for .git directory
        let git_dir = expanded_path.join(".git");
        if !git_dir.exists() {
            self.path
                .set_error("Not a git repository (no .git directory)");
            self.repo_name = None;
            return;
        }

        // Extract repository name from path
        self.repo_name = expanded_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        self.path.set_valid();
    }

    /// Get the expanded path
    pub fn expanded_path(&self) -> PathBuf {
        self.path.expanded_path()
    }

    /// Check if dialog is visible
    pub fn is_visible(&self) -> bool {
        self.path.is_visible()
    }

    /// Validation error message for the active tab
    pub fn error(&self) -> Option<&str> {
        match self.mode {
            AddRepoMode::AddExisting => self.path.error.as_deref(),
            AddRepoMode::CreateNew => self.new_error.as_deref(),
        }
    }

    /// Whether the active tab's input is valid
    pub fn is_valid(&self) -> bool {
        match self.mode {
            AddRepoMode::AddExisting => self.path.is_valid,
            AddRepoMode::CreateNew => self.new_project_path.is_some(),
        }
    }
}

/// Add repository dialog widget
pub struct AddRepoDialog;

impl AddRepoDialog {
    pub fn new() -> Self {
        Self
    }

    /// Render the dialog
    pub fn render(&self, area: Rect, buf: &mut Buffer, state: &AddRepoDialogState) {
        if !state.is_visible() {
            return;
        }

        // Instructions differ per tab (they render on the bottom border)
        let instructions = match state.mode {
            AddRepoMode::AddExisting => {
                vec![("Tab", "create new"), ("Enter", "add"), ("Esc", "cancel")]
            }
            AddRepoMode::CreateNew => vec![
                ("Tab", "add existing"),
                ("↑↓", "field"),
                ("Enter", "create"),
                ("Esc", "cancel"),
            ],
        };

        // Tall enough for the create-new tab (two labelled fields + status), and
        // the same height for both tabs so switching does not resize the dialog.
        // The frame returns a content area that is already padded by one row.
        let frame = DialogFrame::new("Add Custom Project", 60, 14).instructions(instructions);
        let inner = frame.render(area, buf);

        let chunks = Layout::vertical([
            Constraint::Length(1), // Tab bar
            Constraint::Length(1), // Spacing
            Constraint::Min(0),    // Tab body
        ])
        .split(inner);

        self.render_tabs(chunks[0], buf, state.mode);

        match state.mode {
            AddRepoMode::AddExisting => self.render_existing_tab(chunks[2], buf, state),
            AddRepoMode::CreateNew => self.render_new_tab(chunks[2], buf, state),
        }
    }

    /// Render the tab headers, highlighting the active one
    fn render_tabs(&self, area: Rect, buf: &mut Buffer, mode: AddRepoMode) {
        let active = Style::default()
            .fg(accent_primary())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let inactive = Style::default().fg(Color::DarkGray);

        let (existing_style, new_style) = match mode {
            AddRepoMode::AddExisting => (active, inactive),
            AddRepoMode::CreateNew => (inactive, active),
        };

        let line = Line::from(vec![
            Span::styled("Add existing", existing_style),
            Span::raw("    "),
            Span::styled("Create new project", new_style),
        ]);
        Paragraph::new(line).render(area, buf);
    }

    /// Render the "Add existing" tab: one path field
    fn render_existing_tab(&self, area: Rect, buf: &mut Buffer, state: &AddRepoDialogState) {
        let chunks = Layout::vertical([
            Constraint::Length(1), // Label
            Constraint::Length(3), // Input field (with border)
            Constraint::Length(1), // Status/error
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

        let label =
            Paragraph::new("Enter local repository path:").style(Style::default().fg(Color::White));
        label.render(chunks[0], buf);

        self.render_field(
            chunks[1],
            buf,
            &state.path.text,
            true,
            "~/path/to/repo",
            validation_style(state),
        );

        let success_msg = format!(
            "Valid repository: {}",
            state.repo_name.as_deref().unwrap_or("repository")
        );
        let status = StatusLine::from_result(state.error(), state.is_valid(), &success_msg);
        status.render(chunks[2], buf);
    }

    /// Render the "Create new project" tab: name + parent folder
    fn render_new_tab(&self, area: Rect, buf: &mut Buffer, state: &AddRepoDialogState) {
        let chunks = Layout::vertical([
            Constraint::Length(1), // Name label
            Constraint::Length(3), // Name input
            Constraint::Length(1), // Parent label
            Constraint::Length(3), // Parent input
            Constraint::Length(1), // Status/preview
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

        let name_focused = state.new_focus == NewProjectField::Name;
        let parent_focused = !name_focused;

        Paragraph::new("Project name:")
            .style(Style::default().fg(Color::White))
            .render(chunks[0], buf);
        self.render_field(
            chunks[1],
            buf,
            &state.new_name,
            name_focused,
            "my-app",
            if name_focused {
                validation_style(state)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        );

        Paragraph::new("Parent folder:")
            .style(Style::default().fg(Color::White))
            .render(chunks[2], buf);
        self.render_field(
            chunks[3],
            buf,
            &state.new_parent,
            parent_focused,
            "~/projects",
            if parent_focused {
                validation_style(state)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        );

        // When valid, the status line doubles as a preview of what gets created
        let success_msg = match state.new_project_preview() {
            Some(path) => format!("Will create: {}", path),
            None => String::new(),
        };
        let status = StatusLine::from_result(state.error(), state.is_valid(), &success_msg);
        status.render(chunks[4], buf);
    }

    /// Render one bordered text field; the cursor only shows when focused
    fn render_field(
        &self,
        area: Rect,
        buf: &mut Buffer,
        input: &TextInputState,
        focused: bool,
        placeholder: &str,
        border_style: Style,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(border_style);

        let field_inner = block.inner(area);
        block.render(area, buf);

        if focused {
            input.render_with_placeholder(
                field_inner,
                buf,
                Style::default().fg(Color::White),
                placeholder,
                Style::default().fg(Color::DarkGray),
            );
        } else if input.is_empty() {
            Paragraph::new(placeholder)
                .style(Style::default().fg(Color::DarkGray))
                .render(field_inner, buf);
        } else {
            Paragraph::new(input.value())
                .style(Style::default().fg(Color::Gray))
                .render(field_inner, buf);
        }
    }
}

/// Border color for the field being validated
fn validation_style(state: &AddRepoDialogState) -> Style {
    if state.is_valid() {
        Style::default().fg(Color::Green)
    } else if state.error().is_some() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::White)
    }
}

impl Default for AddRepoDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn type_text(state: &mut AddRepoDialogState, text: &str) {
        for c in text.chars() {
            state.insert_char(c);
        }
    }

    #[test]
    fn starts_on_the_existing_tab_and_toggles() {
        let mut state = AddRepoDialogState::new();
        assert_eq!(state.mode, AddRepoMode::AddExisting);

        state.toggle_mode();
        assert_eq!(state.mode, AddRepoMode::CreateNew);

        state.toggle_mode();
        assert_eq!(state.mode, AddRepoMode::AddExisting);
    }

    #[test]
    fn typing_targets_the_focused_field() {
        let mut state = AddRepoDialogState::new();

        type_text(&mut state, "/tmp/repo");
        assert_eq!(state.input(), "/tmp/repo");

        state.toggle_mode();
        type_text(&mut state, "my-app");
        assert_eq!(state.new_project_name(), "my-app");
        assert!(state.new_parent.is_empty());

        state.focus_next_field();
        type_text(&mut state, "~/projects");
        assert_eq!(state.new_parent.value(), "~/projects");
        // Switching fields must not disturb the other one
        assert_eq!(state.new_project_name(), "my-app");
        // Nor the existing tab's path
        assert_eq!(state.input(), "/tmp/repo");
    }

    #[test]
    fn field_focus_does_not_move_on_the_existing_tab() {
        let mut state = AddRepoDialogState::new();
        state.focus_next_field();
        assert_eq!(state.new_focus, NewProjectField::Name);
    }

    #[test]
    fn new_project_is_invalid_until_both_fields_are_filled() {
        let dir = tempdir().unwrap();
        let mut state = AddRepoDialogState::new();
        state.toggle_mode();

        // Nothing typed: not valid, but no error shown either
        state.validate();
        assert!(!state.is_valid());
        assert_eq!(state.error(), None);

        type_text(&mut state, "my-app");
        assert!(!state.is_valid());
        assert_eq!(state.error(), None);

        state.focus_next_field();
        type_text(&mut state, dir.path().to_str().unwrap());
        assert!(state.is_valid());
        assert_eq!(state.new_project_path, Some(dir.path().join("my-app")));
    }

    #[test]
    fn new_project_reports_existing_target() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("my-app")).unwrap();

        let mut state = AddRepoDialogState::new();
        state.toggle_mode();
        type_text(&mut state, "my-app");
        state.focus_next_field();
        type_text(&mut state, dir.path().to_str().unwrap());

        assert!(!state.is_valid());
        assert!(state.error().unwrap().contains("already exists"));
    }

    #[test]
    fn new_project_reports_missing_parent() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nope");

        let mut state = AddRepoDialogState::new();
        state.toggle_mode();
        type_text(&mut state, "my-app");
        state.focus_next_field();
        type_text(&mut state, missing.to_str().unwrap());

        assert!(!state.is_valid());
        assert!(state.error().unwrap().contains("does not exist"));
    }

    #[test]
    fn preview_shows_the_target_path() {
        let mut state = AddRepoDialogState::new();
        state.toggle_mode();
        assert_eq!(state.new_project_preview(), None);

        type_text(&mut state, "my-app");
        state.focus_next_field();
        type_text(&mut state, "~/projects/");

        assert_eq!(
            state.new_project_preview(),
            Some("~/projects/my-app".to_string())
        );
    }

    /// Render the dialog into a buffer and return it as text lines
    fn render_to_lines(state: &AddRepoDialogState) -> Vec<String> {
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        AddRepoDialog::new().render(area, &mut buf, state);

        (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn renders_the_create_new_tab() {
        let dir = tempdir().unwrap();
        let parent = dir.path().to_str().unwrap().to_string();

        let mut state = AddRepoDialogState::new();
        state.show();
        state.toggle_mode();
        type_text(&mut state, "my-app");
        state.focus_next_field();
        type_text(&mut state, &parent);

        let rendered = render_to_lines(&state).join("\n");

        assert!(rendered.contains("Add existing"), "{rendered}");
        assert!(rendered.contains("Create new project"), "{rendered}");
        assert!(rendered.contains("Project name:"), "{rendered}");
        assert!(rendered.contains("Parent folder:"), "{rendered}");
        // Both fields must be wide enough to show what was typed: a layout that
        // squeezes a field to its borders would hide the text.
        assert!(rendered.contains("my-app"), "{rendered}");
        // The preview line must not be cut off by the dialog height
        assert!(rendered.contains("Will create:"), "{rendered}");
    }

    #[test]
    fn renders_the_existing_tab_with_its_field() {
        let mut state = AddRepoDialogState::new();
        state.show();
        type_text(&mut state, "/tmp/some-repo");

        let rendered = render_to_lines(&state).join("\n");

        assert!(
            rendered.contains("Enter local repository path:"),
            "{rendered}"
        );
        assert!(rendered.contains("/tmp/some-repo"), "{rendered}");
    }

    #[test]
    fn show_resets_both_tabs() {
        let mut state = AddRepoDialogState::new();
        state.toggle_mode();
        type_text(&mut state, "my-app");
        state.focus_next_field();

        state.show();

        assert_eq!(state.mode, AddRepoMode::AddExisting);
        assert_eq!(state.new_focus, NewProjectField::Name);
        assert!(state.new_name.is_empty());
        assert!(state.new_parent.is_empty());
        assert!(state.new_project_path.is_none());
        assert_eq!(state.error(), None);
    }
}
