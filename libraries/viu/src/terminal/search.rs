// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded grid search. Coordinates belong to the current native grid; Mix
//! owns the query, navigation and presentation. No output mirror or selection
//! mutation is needed to find text in retained history or soft-wrapped rows.
use super::Terminal;
use rio_vt::crosswords::{
    grid::Dimensions,
    pos::{Column, Direction, Line, Pos, Side},
    search::RegexSearch,
    square::Wide,
};
use serde::{Deserialize, Serialize};

pub struct SearchPattern(RegexSearch);

impl SearchPattern {
    /// Reuse Rio's bounded DFA construction. Explicit flags avoid inheriting
    /// its smart-case default when the caller has chosen case sensitivity.
    pub fn new(
        query: &str,
        regular_expression: bool,
        case_sensitive: bool,
    ) -> Result<Self, String> {
        if query.is_empty() || query.len() > 4096 {
            return Err("search query must contain 1..4096 UTF-8 bytes".into());
        }
        let pattern = if regular_expression {
            query.to_owned()
        } else {
            regex::escape(query)
        };
        let flag = if case_sensitive { "-i" } else { "i" };
        RegexSearch::new(&format!("(?{flag}:{pattern})"))
            .map(Self)
            .map_err(|error| format!("invalid search expression: {error}"))
    }
}

/// Inclusive cell coordinates: negative rows address retained scrollback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SearchPoint {
    pub row: i32,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SearchRange {
    pub start: SearchPoint,
    pub end: SearchPoint,
}

#[derive(Debug, Serialize)]
pub struct SearchOutcome {
    pub matched: Option<SearchRange>,
    pub wrapped: bool,
    /// A no-match in a bounded window is not proof the entire history lacks it.
    pub limited: bool,
}

#[derive(Clone, Copy)]
pub struct SearchRequest {
    pub origin: Option<SearchPoint>,
    pub backwards: bool,
    pub max_lines: usize,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            origin: None,
            backwards: false,
            max_lines: 10000,
        }
    }
}

/// Supplied by the current widget theme; never cached in the VT grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchColors {
    pub foreground: [u8; 3],
    pub background: [u8; 3],
}

impl SearchRange {
    pub(super) fn contains(&self, row: i32, col: usize) -> bool {
        (self.start.row, self.start.col) <= (row, col) && (row, col) <= (self.end.row, self.end.col)
    }
}

impl Terminal {
    /// Search the live Rio grid under its lock, leaving selection, scroll and
    /// PTY input untouched. Results are observations, not persistent anchors:
    /// output/resize/eviction can move them before a later presentation step.
    pub fn search(
        &self,
        pattern: &mut SearchPattern,
        origin: Option<SearchPoint>,
        backwards: bool,
        max_lines: usize,
    ) -> Result<SearchOutcome, String> {
        pattern.find_grid(&self.grid.lock(), origin, backwards, max_lines)
    }
}

impl SearchPattern {
    pub(super) fn find_grid(
        &mut self,
        term: &rio_vt::crosswords::Crosswords<super::Listener>,
        origin: Option<SearchPoint>,
        backwards: bool,
        max_lines: usize,
    ) -> Result<SearchOutcome, String> {
        if !(1..=10000).contains(&max_lines) {
            return Err("search line budget must be 1..10000".into());
        }
        let origin = origin.unwrap_or_else(|| SearchPoint {
            row: -(term.display_offset() as i32)
                + if backwards {
                    term.screen_lines() as i32 - 1
                } else {
                    0
                },
            col: if backwards { term.columns() - 1 } else { 0 },
        });
        if origin.row < term.grid.topmost_line().0
            || origin.row > term.grid.bottommost_line().0
            || origin.col >= term.columns()
        {
            return Err("search origin is outside the retained grid".into());
        }
        let position = Pos::new(Line(origin.row), Column(origin.col));
        let found = term.search_next(
            &mut self.0,
            position,
            if backwards {
                Direction::Left
            } else {
                Direction::Right
            },
            if backwards { Side::Right } else { Side::Left },
            Some(max_lines),
        );
        let point = |p: &Pos| SearchPoint {
            row: p.row.0,
            col: p.col.0,
        };
        let matched = found.map(|range| {
            let mut start = *range.start();
            // Rio's search iterator can anchor a wrapped wide glyph at the
            // preceding row's LeadingSpacer. Report its actual glyph lead so
            // a caller can navigate/highlight without selecting a blank cell.
            if term.grid[start].wide() == Wide::LeadingSpacer
                && start.row < term.grid.bottommost_line()
            {
                let next = Pos::new(start.row + 1i32, Column(0));
                if next <= *range.end() && term.grid[next].wide() == Wide::Wide {
                    start = next;
                }
            }
            SearchRange {
                start: point(&start),
                end: point(range.end()),
            }
        });
        let wrapped = matched.as_ref().is_some_and(|range| {
            if backwards {
                (range.end.row, range.end.col) > (origin.row, origin.col)
            } else {
                (range.start.row, range.start.col) < (origin.row, origin.col)
            }
        });
        Ok(SearchOutcome {
            matched,
            wrapped,
            limited: max_lines < term.grid.total_lines(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_moves_clears_and_rethemes_without_changing_selection() {
        let t = Terminal::from_test_vt(12, 3, b"hit\r\nmiss\r\nhit");
        t.selection_start(
            0,
            1,
            super::super::SelectionSide::Left,
            super::super::SelectionType::Simple,
        );
        t.selection_update(3, 1, super::super::SelectionSide::Right);
        let selected = t.selection_text();
        let plain = t.grid_snapshot().screen;
        let colors = SearchColors {
            foreground: [10, 20, 30],
            background: [200, 100, 50],
        };
        let mut pattern = SearchPattern::new("hit", false, true).unwrap();
        let (first, outcome) = t
            .grid_snapshot_with_search(&mut pattern, SearchRequest::default(), colors)
            .unwrap();
        assert_eq!(outcome.matched.unwrap().start.row, 0);
        assert_eq!(first.dirty_rows, [true, false, false]);
        assert_eq!(first.screen.cells[0].bg, colors.background);
        assert_eq!(first.screen.cells[0].fg, colors.foreground);
        assert_eq!(first.screen.cells[3].bg, plain.cells[3].bg);
        let (idle, _) = t
            .grid_snapshot_with_search(&mut pattern, SearchRequest::default(), colors)
            .unwrap();
        assert!(idle.dirty_rows.iter().all(|dirty| !dirty));
        let request = SearchRequest {
            origin: Some(SearchPoint { row: 2, col: 0 }),
            ..Default::default()
        };
        let (moved, _) = t
            .grid_snapshot_with_search(&mut pattern, request, colors)
            .unwrap();
        assert_eq!(moved.dirty_rows, [true, false, true]);
        assert_eq!(moved.screen.cells[0].bg, plain.cells[0].bg);
        assert_eq!(moved.screen.cells[24].bg, colors.background);
        let changed = SearchColors {
            background: [30, 40, 50],
            ..colors
        };
        let (rethemed, _) = t
            .grid_snapshot_with_search(&mut pattern, request, changed)
            .unwrap();
        assert_eq!(rethemed.dirty_rows, [false, false, true]);
        assert_eq!(rethemed.screen.cells[24].bg, changed.background);
        let cleared = t.grid_snapshot();
        assert_eq!(cleared.dirty_rows, [false, false, true]);
        for (actual, expected) in cleared.screen.cells.iter().zip(&plain.cells) {
            assert_eq!(
                (actual.c, actual.fg, actual.bg),
                (expected.c, expected.fg, expected.bg)
            );
        }
        assert_eq!(t.selection_text(), selected);
        assert_eq!(t.display_offset(), 0);
    }

    #[test]
    fn highlight_covers_both_wide_halves_but_not_the_leading_spacer() {
        let t = Terminal::from_test_vt(5, 3, "abcd中z".as_bytes());
        let plain = t.grid_snapshot().screen;
        let colors = SearchColors {
            foreground: [10, 20, 30],
            background: [200, 100, 50],
        };
        let mut pattern = SearchPattern::new("中", false, true).unwrap();
        let (frame, _) = t
            .grid_snapshot_with_search(&mut pattern, SearchRequest::default(), colors)
            .unwrap();
        assert_eq!(frame.screen.cells[4].bg, plain.cells[4].bg);
        for index in [5, 6] {
            assert_eq!(frame.screen.cells[index].bg, colors.background);
            assert_eq!(frame.screen.cells[index].fg, colors.foreground);
        }
        assert_eq!(frame.screen.cells[7].bg, plain.cells[7].bg);
    }

    #[test]
    fn literal_regex_case_and_reverse_navigation_use_real_cells() {
        let t = Terminal::from_test_vt(20, 3, b"A[1] A11 a[1]");
        let mut literal = SearchPattern::new("A[1]", false, true).unwrap();
        let first = t.search(&mut literal, None, false, 10000).unwrap();
        assert_eq!(
            first.matched.unwrap(),
            SearchRange {
                start: SearchPoint { row: 0, col: 0 },
                end: SearchPoint { row: 0, col: 3 }
            }
        );
        assert!(!first.wrapped);
        let mut insensitive = SearchPattern::new("A[1]", false, false).unwrap();
        let second = t
            .search(
                &mut insensitive,
                Some(SearchPoint { row: 0, col: 5 }),
                false,
                10000,
            )
            .unwrap();
        assert_eq!(second.matched.unwrap().start.col, 9);
        let wrapped = t
            .search(
                &mut literal,
                Some(SearchPoint { row: 0, col: 5 }),
                false,
                10000,
            )
            .unwrap();
        assert!(wrapped.wrapped);
        assert_eq!(wrapped.matched.unwrap().start.col, 0);
        let mut expression = SearchPattern::new("A[0-9]+", true, true).unwrap();
        let found = t
            .search(
                &mut expression,
                Some(SearchPoint { row: 0, col: 12 }),
                true,
                10000,
            )
            .unwrap();
        assert_eq!(
            found.matched.unwrap(),
            SearchRange {
                start: SearchPoint { row: 0, col: 5 },
                end: SearchPoint { row: 0, col: 7 }
            }
        );
    }

    #[test]
    fn history_and_wide_wrapped_search_do_not_change_selection_or_scroll() {
        let t = Terminal::from_test_vt(12, 2, b"old\r\nmiddle\r\nlast");
        t.selection_start(
            0,
            0,
            super::super::SelectionSide::Left,
            super::super::SelectionType::Simple,
        );
        t.selection_update(2, 0, super::super::SelectionSide::Right);
        let selected = t.selection_text();
        let mut pattern = SearchPattern::new("old", false, true).unwrap();
        let result = t
            .search(
                &mut pattern,
                Some(SearchPoint { row: -1, col: 0 }),
                false,
                10000,
            )
            .unwrap();
        assert_eq!(
            result.matched.unwrap().start,
            SearchPoint { row: -1, col: 0 }
        );
        assert_eq!(t.display_offset(), 0);
        assert_eq!(t.selection_text(), selected);
        assert!(t.search(&mut pattern, None, false, 1).unwrap().limited);
        let wide = Terminal::from_test_vt(5, 3, "abcd中z".as_bytes());
        assert_eq!(
            wide.grid.lock().grid[Pos::new(Line(0), Column(4))].wide(),
            Wide::LeadingSpacer
        );
        assert_eq!(
            wide.grid.lock().grid[Pos::new(Line(1), Column(0))].wide(),
            Wide::Wide
        );
        let mut pattern = SearchPattern::new("中z", false, true).unwrap();
        let result = wide.search(&mut pattern, None, false, 10000).unwrap();
        assert_eq!(
            result.matched.unwrap(),
            SearchRange {
                start: SearchPoint { row: 1, col: 0 },
                end: SearchPoint { row: 1, col: 2 }
            }
        );
    }

    #[test]
    fn malformed_patterns_origins_and_budgets_refuse_without_mutation() {
        assert!(SearchPattern::new("", false, true).is_err());
        assert!(SearchPattern::new(&"a".repeat(4097), false, true).is_err());
        assert!(SearchPattern::new("[", true, true).is_err());
        let t = Terminal::from_test_vt(8, 2, b"untouched");
        let before = t.snapshot();
        let mut pattern = SearchPattern::new("absent", false, true).unwrap();
        for origin in [
            SearchPoint { row: -100, col: 0 },
            SearchPoint { row: 2, col: 0 },
            SearchPoint { row: 0, col: 8 },
        ] {
            assert!(t.search(&mut pattern, Some(origin), false, 10000).is_err());
        }
        assert!(t.search(&mut pattern, None, false, 0).is_err());
        assert!(t.search(&mut pattern, None, false, 10001).is_err());
        assert!(
            t.search(&mut pattern, None, false, 10000)
                .unwrap()
                .matched
                .is_none()
        );
        assert_eq!(t.snapshot(), before);
    }

    #[test]
    fn empty_regex_matches_near_wide_spacers_never_invert_the_range() {
        let t = Terminal::from_test_vt(5, 3, "abcd中z".as_bytes());
        for query in ["a*", ".*?"] {
            let mut pattern = SearchPattern::new(query, true, true).unwrap();
            for backwards in [false, true] {
                for col in 0..5 {
                    let result = t
                        .search(
                            &mut pattern,
                            Some(SearchPoint { row: 0, col }),
                            backwards,
                            10000,
                        )
                        .unwrap();
                    if let Some(range) = result.matched {
                        assert!(
                            (range.start.row, range.start.col) <= (range.end.row, range.end.col)
                        );
                    }
                }
            }
        }
    }
}
