/// Scan a byte buffer for `ESC ] 1337 ; LazyjustDone=<int> BEL` sequences.
/// Returns the bytes with those sequences removed and a list of captured exit codes.
pub fn scan_done_marker(input: &[u8]) -> (Vec<u8>, Vec<i32>) {
    let prefix = b"\x1b]1337;LazyjustDone=";
    let mut out = Vec::with_capacity(input.len());
    let mut codes = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(prefix) {
            if let Some(bell_rel) = input[i + prefix.len()..].iter().position(|&b| b == 0x07) {
                let num_slice = &input[i + prefix.len()..i + prefix.len() + bell_rel];
                if let Ok(s) = std::str::from_utf8(num_slice) {
                    if let Ok(code) = s.parse::<i32>() {
                        codes.push(code);
                        i += prefix.len() + bell_rel + 1;
                        continue;
                    }
                }
            }
        }
        out.push(input[i]);
        i += 1;
    }
    (out, codes)
}

/// The DSR cursor-position query: `ESC [ 6 n`.
const CURSOR_QUERY: &[u8] = b"\x1b[6n";

/// Split `input` on DSR cursor-position queries, dropping the query bytes.
///
/// The returned vector always has one more element than the number of
/// queries found, so `chunks.len() - 1` is the query count and every gap
/// between consecutive chunks marks a point where a report is owed.
///
/// Why lazyjust must answer: the session PTY runs `$SHELL -i`, so the
/// user's rc files run before the primed recipe line. Tools that draw
/// inline images (fastfetch, p10k's instant prompt) emit `ESC [ 6 n` and
/// then block reading stdin for the `ESC [ row ; col R` report. Nothing
/// else in the stack replies — `vt100` has no DSR support at all — so the
/// probe never returns and swallows the primed recipe line as it hunts
/// for the report. The recipe never runs and the pane stays blank.
///
/// Caveat: a query straddling two PTY reads is not reassembled, matching
/// the existing limitation in [`scan_done_marker`]. The sequence is four
/// bytes and the reads are 8 KiB, so a split is possible but rare.
pub fn split_cursor_queries(input: &[u8]) -> Vec<&[u8]> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(CURSOR_QUERY) {
            chunks.push(&input[start..i]);
            i += CURSOR_QUERY.len();
            start = i;
        } else {
            i += 1;
        }
    }
    chunks.push(&input[start..]);
    chunks
}

/// Build the DSR report for a cursor at `row`/`col`. Both inputs are
/// 0-based (as `vt100::Screen::cursor_position` returns them); the wire
/// format is 1-based.
pub fn cursor_report(row: u16, col: u16) -> Vec<u8> {
    format!("\x1b[{};{}R", row + 1, col + 1).into_bytes()
}
