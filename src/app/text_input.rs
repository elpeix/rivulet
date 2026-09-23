use std::fmt;
use std::ops::Deref;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    value: String,
    cursor: usize,
}

impl TextInput {
    #[cfg(test)]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.char_count();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn insert(&mut self, c: char) {
        let index = self.byte_index(self.cursor);
        self.value.insert(index, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let index = self.byte_index(self.cursor);
        self.value.remove(index);
    }

    pub fn delete(&mut self) {
        if self.cursor < self.char_count() {
            let index = self.byte_index(self.cursor);
            self.value.remove(index);
        }
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.char_count());
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.char_count();
    }

    pub fn split_at_cursor(&self) -> (&str, Option<char>, &str) {
        let index = self.byte_index(self.cursor);
        let (before, rest) = self.value.split_at(index);
        let mut chars = rest.chars();
        let current = chars.next();
        (before, current, chars.as_str())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Home => self.move_home(),
            KeyCode::End => self.move_end(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => self.insert(c),
            _ => return false,
        }
        true
    }

    fn char_count(&self) -> usize {
        self.value.chars().count()
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_index)
            .map_or(self.value.len(), |(index, _)| index)
    }
}

impl Deref for TextInput {
    type Target = str;

    fn deref(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for TextInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl PartialEq<&str> for TextInput {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn input(value: &str) -> TextInput {
        let mut input = TextInput::default();
        input.set(value);
        input
    }

    #[test]
    fn set_places_cursor_at_end() {
        let input = input("abc");
        assert_eq!(input, "abc");
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn clear_empties_value_and_resets_cursor() {
        let mut input = input("abc");
        input.clear();
        assert_eq!(input, "");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn insert_at_cursor_in_the_middle() {
        let mut input = input("ac");
        input.move_left();
        input.insert('b');
        assert_eq!(input, "abc");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn backspace_removes_char_before_cursor() {
        let mut input = input("abc");
        input.move_left();
        input.backspace();
        assert_eq!(input, "ac");
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn backspace_at_start_does_nothing() {
        let mut input = input("abc");
        input.move_home();
        input.backspace();
        assert_eq!(input, "abc");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn delete_removes_char_under_cursor() {
        let mut input = input("abc");
        input.move_home();
        input.delete();
        assert_eq!(input, "bc");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn delete_at_end_does_nothing() {
        let mut input = input("abc");
        input.delete();
        assert_eq!(input, "abc");
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn cursor_movement_is_clamped() {
        let mut input = input("ab");
        input.move_right();
        assert_eq!(input.cursor(), 2);
        input.move_home();
        input.move_left();
        assert_eq!(input.cursor(), 0);
        input.move_end();
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn multibyte_chars_are_edited_as_single_chars() {
        let mut input = input("ça và");
        input.move_left();
        input.backspace();
        assert_eq!(input, "ça à");
        input.move_home();
        input.delete();
        assert_eq!(input, "a à");
        input.insert('é');
        assert_eq!(input, "éa à");
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn split_at_cursor_returns_before_current_and_after() {
        let mut input = input("abc");
        input.move_left();
        input.move_left();
        assert_eq!(input.split_at_cursor(), ("a", Some('b'), "c"));
        input.move_end();
        assert_eq!(input.split_at_cursor(), ("abc", None, ""));
    }

    #[test]
    fn handle_key_edits_and_moves() {
        let mut input = input("htp://x");
        assert!(input.handle_key(key(KeyCode::Home)));
        assert!(input.handle_key(key(KeyCode::Right)));
        assert!(input.handle_key(key(KeyCode::Char('t'))));
        assert_eq!(input, "http://x");
        assert!(input.handle_key(key(KeyCode::End)));
        assert!(input.handle_key(key(KeyCode::Left)));
        assert!(input.handle_key(key(KeyCode::Delete)));
        assert!(input.handle_key(key(KeyCode::Backspace)));
        assert_eq!(input, "http:/");
    }

    #[test]
    fn handle_key_ignores_control_chars_and_other_keys() {
        let mut input = input("abc");
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(!input.handle_key(ctrl_c));
        assert!(!input.handle_key(key(KeyCode::Tab)));
        assert_eq!(input, "abc");
    }
}
