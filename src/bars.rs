use std::io::Write;
use tokio::sync::mpsc;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const DIM: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

/// Real TTY columns; falls back to $COLUMNS then 80 (e.g. when piped).
pub fn terminal_width() -> usize {
    if let Some((w, _)) = terminal_size::terminal_size() {
        return w.0 as usize;
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80)
}

/// Width of the fixed right part: "{recv} | {loss} | {total}".
fn counter_width(count: u16) -> usize {
    format!("{:>4} |{:>4} |{:>4}", 0, 0, count).len()
}

/// Width of one ping's segment if the whole bar fits into `max_width`.
pub fn segment_width(term_width: usize, count: u16) -> usize {
    if count == 0 {
        return 1;
    }
    (term_width / count as usize).max(1)
}

/// Bar with ANSI colors: green # per reply, red # per loss, dim dots for pending pings.
/// If it would exceed `max_width`, segments shrink to 1 char and the earliest pings are dropped.
pub fn build_bar(state: &[bool], count: u16, seg: usize, max_width: usize) -> String {
    let mut seg = seg.max(1);
    let mut total = count as usize;
    if seg * total > max_width {
        seg = 1;
        total = total.min(max_width);
    }

    let offset = count as usize - total; // earliest pings not shown
    let mut bar = String::new();
    for i in 0..total {
        let (color, ch) = match state.get(i + offset) {
            Some(true) => (GREEN, '#'),
            Some(false) => (RED, '#'),
            None => (DIM, '·'),
        };
        bar.push_str(color);
        for _ in 0..seg {
            bar.push(ch);
        }
    }
    bar.push_str(RESET);
    bar
}

/// Live renderer: one row per host, redrawn in place on every ping result.
/// Terminal width is re-read on each redraw so window resizes don't corrupt output.
pub async fn run(
    labels: Vec<String>,
    count: u16,
    mut rx: mpsc::UnboundedReceiver<(usize, bool)>,
) {
    let label_width = labels.iter().map(|l| l.len()).max().unwrap_or(0).max(4);
    let mut state: Vec<Vec<bool>> = vec![vec![]; labels.len()];
    let mut printed_rows = 0;

    while let Some((idx, ok)) = rx.recv().await {
        if idx < state.len() {
            state[idx].push(ok);
        }

        let term_width = terminal_width();
        let max_bar_width = term_width
            .saturating_sub(label_width + 1 + counter_width(count) + 1)
            .max(1);
        let seg = segment_width(term_width, count);
        let mut out = std::io::stdout().lock();
        if printed_rows > 0 {
            let _ = write!(out, "\x1b[{}A", printed_rows);
        }
        printed_rows = 0;
        for (i, label) in labels.iter().enumerate() {
            let bar = build_bar(&state[i], count, seg, max_bar_width);
            let (recv, loss) = (state[i].iter().filter(|r| **r).count(), state[i].iter().filter(|r| !**r).count());
            let _ = writeln!(out, "{:<w$} {} {:>4} |{:>4} |{:>4}", label, bar, recv, loss, count, w = label_width);
            printed_rows += 1;
        }
        let _ = out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_width_splits_terminal_over_pings() {
        assert_eq!(segment_width(80, 10), 8);
        assert_eq!(segment_width(10, 100), 1);
        assert_eq!(segment_width(80, 0), 1);
    }

    #[test]
    fn bar_colors_results_and_pads_pending() {
        let state = vec![true, false];
        let bar = build_bar(&state, 3, 2, 80);
        assert_eq!(bar, format!("{GREEN}##{RED}##{DIM}··{RESET}"));
    }

    #[test]
    fn bar_shrinks_to_one_char_segments_when_too_wide() {
        let state = vec![true, false, true];
        let bar = build_bar(&state, 3, 20, 3);
        // 3 chars fit: all pings shown at 1 char each
        assert_eq!(bar, format!("{GREEN}#{RED}#{GREEN}#{RESET}"));
    }

    #[test]
    fn bar_caps_at_max_width_when_more_pings_than_chars() {
        let state = vec![true, false, true, true, true, false];
        let bar = build_bar(&state, 6, 20, 4);
        // count 6 > max 4: earliest 2 pings dropped, 1 char each remains
        assert_eq!(bar, format!("{GREEN}#{GREEN}#{GREEN}#{RED}#{RESET}"));
    }
}
