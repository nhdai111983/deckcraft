//! Home › Editing: Find and Replace across slide text, table cells and speaker notes.

use std::sync::Arc;

use deckcraft_model::text::RunKind;
use deckcraft_model::{Paragraph, Shape, ShapeKind, Slide, TextBody};
use serde_json::{Value, json};

use super::*;
use crate::{Result, Session};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "text.find",
            "Find",
            ["Home", "Editing"],
            None,
            "{text, matchCase?: bool, wholeWords?: bool} → [{slide, shape (null = notes), cell?: [row, col], para, start, end}] (char offsets in the paragraph)",
            has_doc,
            find
        ),
        cmd!(
            "text.replace",
            "Replace",
            ["Home", "Editing"],
            None,
            "{text, with, matchCase?, wholeWords?, at?: a text.find match (replace just that one; default: replace all)} → {replaced}",
            has_doc,
            replace
        ),
    ]
}

/// Where a text body lives on a slide.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Loc {
    shape: Option<u64>,
    cell: Option<(usize, usize)>,
}

/// Visit every text body on a slide: shapes depth-first (groups, table cells), then the notes.
fn bodies(slide: &mut Slide, f: &mut dyn FnMut(Loc, &mut TextBody)) {
    fn rec(shapes: &mut [Shape], depth: usize, f: &mut dyn FnMut(Loc, &mut TextBody)) {
        if depth > 64 {
            return;
        }
        for sh in shapes {
            let id = Some(u64::from(sh.id.0));
            if let Some(t) = &mut sh.text {
                f(Loc { shape: id, cell: None }, t);
            }
            match &mut sh.kind {
                ShapeKind::Table(t) => {
                    for (r, row) in t.rows.iter_mut().enumerate() {
                        for (c, cell) in row.cells.iter_mut().enumerate() {
                            f(Loc { shape: id, cell: Some((r, c)) }, &mut cell.text);
                        }
                    }
                }
                ShapeKind::Group { children, .. } => rec(children, depth + 1, f),
                _ => {}
            }
        }
    }
    rec(&mut slide.shapes, 0, f);
    f(Loc { shape: None, cell: None }, &mut slide.notes);
}

struct Opts {
    /// The text to find, one key per letter ([`Opts::key`]).
    pat: Vec<String>,
    /// The text to find as typed, when it is combining marks only: such a query looks for the
    /// marks themselves, inside letters too (removing a stray accent).
    marks: Option<Vec<char>>,
    case: bool,
    whole: bool,
}

/// A letter of a paragraph: its char range and its comparison key.
struct Letter {
    start: usize,
    end: usize,
    key: String,
}

/// Whether `c` is a combining mark that belongs to the letter before it (U+0300–U+036F). The
/// same range as `deckcraft_collate`'s own (private) test.
fn is_mark(c: char) -> bool {
    ('\u{300}'..='\u{36f}').contains(&c)
}

/// The letters of `text`, one at a time: each char with the combining marks after it, keyed by
/// [`Opts::key`]. Lazy, so a search holds only the letters it compares, never a key per char of
/// a huge paragraph.
struct Letters<'a> {
    text: &'a [char],
    at: usize,
    case: bool,
}

impl Iterator for Letters<'_> {
    type Item = Letter;
    fn next(&mut self) -> Option<Letter> {
        let start = self.at;
        self.text.get(start)?;
        let mut end = start + 1;
        while self.text.get(end).is_some_and(|&m| is_mark(m)) {
            end += 1;
        }
        self.at = end;
        let letter: String = self.text.get(start..end).unwrap_or_default().iter().collect();
        Some(Letter { start, end, key: Opts::key(&letter, self.case) })
    }
}

/// Whether the char at `i` is part of a word for Whole words: a letter, digit or `_`, or a
/// combining mark on one (a mark after a space or punctuation is not).
fn word_at(text: &[char], i: usize) -> bool {
    let Some(&c) = text.get(i) else { return false };
    if !is_mark(c) {
        return c.is_alphanumeric() || c == '_';
    }
    // The base the mark belongs to: the first non-mark before it.
    text.get(..i).and_then(|before| before.iter().rev().find(|&&b| !is_mark(b))).is_some_and(|b| b.is_alphanumeric() || *b == '_')
}

impl Opts {
    fn from(p: &Value, cmd: &str) -> Result<Self> {
        let text = str_param(p, "text").unwrap_or("");
        if text.is_empty() {
            return Err(bad(cmd, "missing `text`"));
        }
        let case = bool_or(p, "matchCase", false);
        let chars: Vec<char> = text.chars().collect();
        let marks = chars.iter().all(|&c| is_mark(c)).then(|| chars.clone());
        let pat = Letters { text: &chars, at: 0, case }.map(|l| l.key).collect();
        Ok(Opts { pat, marks, case, whole: bool_or(p, "wholeWords", false) })
    }

    /// The key two letters share when they are the same letter: canonical (a letter typed
    /// precomposed, `ệ`, equals it typed as base and marks, `e` U+0323 U+0302, in any order of
    /// marks of different classes), and lowercased unless the search matches case.
    fn key(letter: &str, case: bool) -> String {
        if case { deckcraft_collate::canonical(letter) } else { deckcraft_collate::fold_key(letter) }
    }

    /// Non-overlapping matches as char ranges. Letters are compared whole, so a match never
    /// starts or ends inside a letter (`e` does not find the `e` of a decomposed `ễ`), and the
    /// precomposed and decomposed spellings of a letter match each other. A query of combining
    /// marks only matches those marks as typed, wherever they are.
    fn matches(&self, text: &[char]) -> Vec<(usize, usize)> {
        if let Some(marks) = &self.marks {
            return marks_in(text, marks);
        }
        let n = self.pat.len();
        if n == 0 {
            return vec![];
        }
        let mut out = vec![];
        // The last `n` letters read: the window compared with the query.
        let mut window: std::collections::VecDeque<Letter> = std::collections::VecDeque::with_capacity(n);
        for letter in (Letters { text, at: 0, case: self.case }) {
            window.push_back(letter);
            if window.len() < n {
                continue;
            }
            let (Some(a), Some(b)) = (window.front().map(|l| l.start), window.back().map(|l| l.end)) else { break };
            let hit = window.iter().zip(&self.pat).all(|(l, k)| l.key == *k)
                && (!self.whole || (!a.checked_sub(1).is_some_and(|j| word_at(text, j)) && !word_at(text, b)));
            if hit {
                out.push((a, b));
                window.clear();
            } else {
                window.pop_front();
            }
        }
        out
    }
}

/// Non-overlapping occurrences of the mark sequence `marks` in `text`, as char ranges.
fn marks_in(text: &[char], marks: &[char]) -> Vec<(usize, usize)> {
    let n = marks.len();
    if n == 0 {
        return vec![];
    }
    let mut out = vec![];
    let mut i: usize = 0;
    while let Some(w) = text.get(i..i.saturating_add(n)) {
        if w == marks {
            out.push((i, i + n));
            i += n;
        } else {
            i += 1;
        }
    }
    out
}

fn para_chars(p: &Paragraph) -> Vec<char> {
    p.runs.iter().flat_map(|r| r.text.chars()).collect()
}

/// Matches that lie wholly in plain text runs (never inside a field, equation or break).
fn para_matches(p: &Paragraph, o: &Opts) -> Vec<(usize, usize)> {
    let mut spans = vec![];
    let mut off = 0;
    for r in &p.runs {
        let n = r.text.chars().count();
        spans.push((off, off + n, matches!(r.kind, RunKind::Text)));
        off += n;
    }
    o.matches(&para_chars(p)).into_iter().filter(|&(a, b)| spans.iter().all(|&(s, e, text)| text || e <= a || s >= b || s == e)).collect()
}

/// Replace chars `a..b` of the paragraph; the replacement takes the first run's formatting.
fn splice(p: &mut Paragraph, a: usize, b: usize, with: &str) {
    let mut off = 0;
    let mut placed = false;
    for r in &mut p.runs {
        let chars: Vec<char> = r.text.chars().collect();
        let (s, e) = (off, off + chars.len());
        off = e;
        if e <= a || s >= b {
            continue;
        }
        let keep_head: String = chars[..a.saturating_sub(s)].iter().collect();
        let keep_tail: String = chars[b.saturating_sub(s).min(chars.len())..].iter().collect();
        r.text = if placed { keep_tail } else { format!("{keep_head}{with}{keep_tail}") };
        placed = true;
    }
    if p.runs.len() > 1 {
        p.runs.retain(|r| !(r.text.is_empty() && matches!(r.kind, RunKind::Text)));
    }
}

fn find(s: &mut Session, p: &Value) -> Result<Value> {
    let o = Opts::from(p, "text.find")?;
    let st = s.doc()?;
    let mut out = vec![];
    for (i, sl) in st.doc.slides.iter().enumerate() {
        let mut sl = (**sl).clone();
        bodies(&mut sl, &mut |loc, t| {
            for (pi, para) in t.paragraphs.iter().enumerate() {
                for (a, b) in para_matches(para, &o) {
                    out.push(json!({"slide": i, "shape": loc.shape, "cell": loc.cell.map(|(r, c)| [r, c]), "para": pi, "start": a, "end": b}));
                }
            }
        });
    }
    Ok(Value::Array(out))
}

fn replace(s: &mut Session, p: &Value) -> Result<Value> {
    let o = Opts::from(p, "text.replace")?;
    let with = str_param(p, "with").unwrap_or("").to_string();
    let at = p.get("at").filter(|v| v.is_object()).map(|v| -> Result<(usize, Loc, usize, usize)> {
        let n = |k: &str| v.get(k).and_then(Value::as_u64).map(|x| x as usize).ok_or_else(|| bad("text.replace", format!("`at` needs `{k}`")));
        let cell = v.get("cell").and_then(Value::as_array).and_then(|c| Some((c.first()?.as_u64()? as usize, c.get(1)?.as_u64()? as usize)));
        Ok((n("slide")?, Loc { shape: v.get("shape").and_then(Value::as_u64), cell }, n("para")?, n("start")?))
    });
    let at = at.transpose()?;
    let count = s.edit(|doc, _| {
        let mut count = 0;
        for (i, slp) in doc.slides.iter_mut().enumerate() {
            if at.is_some_and(|a| a.0 != i) {
                continue;
            }
            let mut sl = (**slp).clone();
            let mut changed = 0;
            bodies(&mut sl, &mut |loc, t| {
                if at.is_some_and(|a| a.1 != loc) {
                    return;
                }
                for (pi, para) in t.paragraphs.iter_mut().enumerate() {
                    if at.is_some_and(|a| a.2 != pi) {
                        continue;
                    }
                    for (a, b) in para_matches(para, &o).into_iter().rev() {
                        if at.is_some_and(|x| x.3 != a) {
                            continue;
                        }
                        splice(para, a, b, &with);
                        changed += 1;
                    }
                }
            });
            if changed > 0 {
                *slp = Arc::new(sl);
                count += changed;
            }
        }
        if count == 0 {
            return Err(bad("text.replace", "no matches"));
        }
        Ok(count)
    })?;
    Ok(json!({"replaced": count}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deckcraft_model::{Run, RunProps};

    fn opts(t: &str, case: bool, whole: bool) -> Opts {
        Opts::from(&json!({"text": t, "matchCase": case, "wholeWords": whole}), "text.find").unwrap()
    }

    #[test]
    fn matching_honours_case_and_whole_words() {
        let t: Vec<char> = "Cat cat category".chars().collect();
        assert_eq!(opts("cat", false, false).matches(&t).len(), 3);
        assert_eq!(opts("cat", true, false).matches(&t).len(), 2);
        assert_eq!(opts("cat", false, true).matches(&t), vec![(0, 3), (4, 7)]);
    }

    #[test]
    fn precomposed_and_decomposed_letters_match_each_other() {
        let text: Vec<char> = "Tiếng Việt, tiê\u{301}ng Viê\u{323}t, tie\u{323}\u{302}ng Vie\u{302}\u{323}t, Viet".chars().collect();
        // Typed precomposed or decomposed, in either mark order: the three spellings of Việt.
        for q in ["Việt", "Vie\u{323}\u{302}t", "Viê\u{323}t"] {
            assert_eq!(opts(q, false, false).matches(&text).len(), 3, "{q:?}");
        }
        assert_eq!(opts("tiếng", false, false).matches(&text).len(), 2);
        // Ranges are the text's own chars: the decomposed match is longer than the query.
        let found = opts("Việt", true, false).matches(&text);
        assert_eq!(found.iter().map(|&(a, b)| b - a).collect::<Vec<_>>(), [4, 5, 6]);
        // Match case in both spellings; whole letters; whole words.
        assert_eq!(opts("VIỆT", true, false).matches(&text).len(), 0);
        assert_eq!(opts("VIỆT", false, false).matches(&text).len(), 3);
        assert_eq!(opts("Vie", false, false).matches(&text).len(), 1);
        assert_eq!(opts("Viê", false, false).matches(&text).len(), 0);
        assert_eq!(opts("viet", false, true).matches(&text).len(), 1);
        // A query of marks only finds those marks as typed, inside letters too (removing a stray
        // accent); a text of marks only never panics.
        assert_eq!(opts("\u{302}", false, false).matches(&text), vec![(30, 31), (37, 38)]);
        assert_eq!(opts("\u{301}", false, false).matches(&"e\u{301}x".chars().collect::<Vec<_>>()), vec![(1, 2)]);
        assert!(opts("a", false, false).matches(&['\u{301}', '\u{301}']).is_empty());
        // Whole words: a mark on a letter is part of the word; a stray mark after a space is not.
        let ww = |q: &str, t: &str| opts(q, false, true).matches(&t.chars().collect::<Vec<_>>());
        assert_eq!(ww("x", "e\u{301}x x"), vec![(4, 5)]);
        assert_eq!(ww("Viet", "x \u{301}Viet"), vec![(3, 7)]);
        // A huge paragraph is searched letter by letter without a key per char kept in memory.
        let big: Vec<char> = "Tiếng Việt ".chars().cycle().take(200_000).collect();
        assert_eq!(opts("Việt", false, false).matches(&big).len(), 200_000 / 11);
    }

    #[test]
    fn splice_across_runs_keeps_the_first_runs_format() {
        let bold = RunProps { bold: Some(true), ..Default::default() };
        let mut p = Paragraph {
            runs: vec![
                Run { text: "Hel".into(), props: bold.clone(), kind: RunKind::Text },
                Run { text: "lo world".into(), props: RunProps::default(), kind: RunKind::Text },
            ],
            ..Default::default()
        };
        splice(&mut p, 1, 5, "ola");
        assert_eq!(para_chars(&p).iter().collect::<String>(), "Hola world");
        assert_eq!(p.runs[0].text, "Hola");
        assert_eq!(p.runs[0].props, bold);
    }
}
