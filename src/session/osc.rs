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
