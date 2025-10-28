//! StatusViewModel state management implementation
//!
//! This module contains all state update and status check methods for managing
//! the interactive state of status items (favorites, reblogs, bookmarks, etc.).

use super::core::StatusViewModel;
use crate::helper::format_number;
use crate::icons::*;

impl StatusViewModel {
    /// Mutate the reply status. This happens when we mutate,
    /// before the backend sends back an update
    #[inline(always)]
    pub fn did_reply(&mut self) {
        self.is_reply = true;
        self.replies_count += 1;
        self.replies = format_number(self.replies_count as i64);
    }

    /// Check reblog status with icon selection
    ///
    /// # Arguments
    /// * `action` - Closure to execute with reblog state and icon
    ///
    /// # Returns
    /// * `T` - Result of the action closure
    ///
    /// # Performance
    /// Uses inline closure for zero-allocation icon selection
    #[inline(always)]
    pub fn is_reblogged<T>(&self, action: impl Fn(bool, &'static str) -> T) -> T {
        let value = self.has_reblogged;
        let icon = if value { ICON_BOOST2 } else { ICON_BOOST1 };
        action(value, icon)
    }

    /// Update reblog count with proper state management
    ///
    /// # Arguments
    /// * `on` - Whether the status is being reblogged (true) or unreblogged (false)
    ///
    /// # Performance
    /// Uses efficient count management with underflow protection
    #[inline(always)]
    pub fn update_reblog(&mut self, on: bool) {
        // See `update_favorited` below for similar pattern
        if !on && self.reblog_count > 0 {
            self.reblog_count -= 1;
        }
        self.reblog = format_number(self.reblog_count as i64);
    }

    /// Check bookmark status with icon selection
    ///
    /// # Arguments
    /// * `action` - Closure to execute with bookmark state and icon
    ///
    /// # Returns
    /// * `T` - Result of the action closure
    ///
    /// # Performance
    /// Uses inline closure for zero-allocation icon selection
    #[inline(always)]
    pub fn is_bookmarked<T>(&self, action: impl Fn(bool, &'static str) -> T) -> T {
        let value = self.is_bookmarked;
        let icon = if value {
            ICON_BOOKMARK2
        } else {
            ICON_BOOKMARK1
        };
        action(value, icon)
    }

    /// Check favourite status with icon selection
    ///
    /// # Arguments
    /// * `action` - Closure to execute with favourite state and icon
    ///
    /// # Returns
    /// * `T` - Result of the action closure
    ///
    /// # Performance
    /// Uses inline closure for zero-allocation icon selection
    #[inline(always)]
    pub fn is_favourited<T>(&self, action: impl Fn(bool, &'static str) -> T) -> T {
        let value = self.is_favourited;
        let icon = if value { ICON_STAR2 } else { ICON_STAR1 };
        action(value, icon)
    }

    /// Update favorite status with proper count management
    ///
    /// # Arguments
    /// * `on` - Whether the status is being favorited (true) or unfavorited (false)
    ///
    /// # Implementation Notes
    /// Uses checked arithmetic to prevent overflow/underflow and ensures
    /// consistent state between is_favourited and favourited_count
    #[inline]
    pub fn update_favorited(&mut self, on: bool) {
        match on {
            true => {
                // Favoriting - increment count and set flag
                self.favourited_count = self.favourited_count.saturating_add(1);
                self.is_favourited = true;
            }
            false => {
                // Unfavoriting - decrement count (with underflow protection) and clear flag
                self.favourited_count = self.favourited_count.saturating_sub(1);
                self.is_favourited = false;
            }
        }

        // Update the formatted string representation
        self.favourited = format_number(self.favourited_count as i64);
    }

    /// Update reblog state with saturating arithmetic for safety
    ///
    /// # Arguments
    /// * `on` - Whether the status is being reblogged (true) or unreblogged (false)
    ///
    /// # Implementation Notes
    /// Uses checked arithmetic to prevent overflow/underflow and ensures
    /// consistent state between has_reblogged and reblog_count
    #[inline]
    pub fn update_reblogged(&mut self, on: bool) {
        match on {
            true => {
                // Reblogging - increment count and set flag
                self.reblog_count = self.reblog_count.saturating_add(1);
                self.has_reblogged = true;
            }
            false => {
                // Unreblogging - decrement count (with underflow protection) and clear flag
                self.reblog_count = self.reblog_count.saturating_sub(1);
                self.has_reblogged = false;
            }
        }

        // Update the formatted string representation
        self.reblog = format_number(self.reblog_count as i64);
        self.favourited = format_number(self.favourited_count as i64);

        // Update the title to reflect current state
        self.favourited_title = format!(
            "Favorites{}",
            if self.is_favourited {
                ": You favourited this"
            } else {
                ""
            }
        );
    }
}
