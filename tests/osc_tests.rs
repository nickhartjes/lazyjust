use lazyjust::session::osc::scan_done_marker;

#[test]
fn finds_marker_and_strips_sequence() {
    let input: &[u8] = b"hello\x1b]1337;LazyjustDone=42\x07world";
    let (remaining, codes) = scan_done_marker(input);
    assert_eq!(codes, vec![42]);
    assert_eq!(remaining, b"helloworld");
}

#[test]
fn multiple_markers() {
    let input: &[u8] = b"a\x1b]1337;LazyjustDone=0\x07b\x1b]1337;LazyjustDone=1\x07c";
    let (remaining, codes) = scan_done_marker(input);
    assert_eq!(codes, vec![0, 1]);
    assert_eq!(remaining, b"abc");
}

#[test]
fn no_marker_passes_through() {
    let input: &[u8] = b"plain bytes";
    let (remaining, codes) = scan_done_marker(input);
    assert!(codes.is_empty());
    assert_eq!(remaining, input);
}

// --- cursor-position (DSR) queries -------------------------------------
//
// An interactive shell's rc files may probe the terminal with
// `ESC [ 6 n` and block until a report comes back. See
// `split_cursor_queries` for why lazyjust has to answer.

use lazyjust::session::osc::{cursor_report, split_cursor_queries};

#[test]
fn no_cursor_query_yields_single_chunk() {
    let chunks = split_cursor_queries(b"plain bytes");
    assert_eq!(chunks, vec![&b"plain bytes"[..]]);
}

#[test]
fn splits_around_cursor_query_and_drops_it() {
    let chunks = split_cursor_queries(b"before\x1b[6nafter");
    assert_eq!(chunks, vec![&b"before"[..], &b"after"[..]]);
}

#[test]
fn splits_on_every_cursor_query() {
    let chunks = split_cursor_queries(b"a\x1b[6nb\x1b[6nc");
    assert_eq!(chunks, vec![&b"a"[..], &b"b"[..], &b"c"[..]]);
}

#[test]
fn cursor_query_at_buffer_edges_yields_empty_chunks() {
    let chunks = split_cursor_queries(b"\x1b[6n");
    assert_eq!(chunks, vec![&b""[..], &b""[..]]);
}

#[test]
fn cursor_report_is_one_based() {
    // vt100 reports a 0-based cursor; the DSR reply is 1-based.
    assert_eq!(cursor_report(0, 0), b"\x1b[1;1R".to_vec());
    assert_eq!(cursor_report(1, 2), b"\x1b[2;3R".to_vec());
}
