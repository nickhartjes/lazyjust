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
