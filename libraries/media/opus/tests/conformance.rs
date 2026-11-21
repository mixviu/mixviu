//! Fixtures: RFC 6716 / RFC 8251, IETF Trust BSD terms. The worker setup checks
//! archive SHA256 before extraction; no vector data is committed here.
use media_opus::{Channels, Decoder, OpusDecoder};
use std::{fs, path::PathBuf, process::Command};
mod support;

#[test]
fn rfc_vectors() {
    let Some(root) = std::env::var_os("MIX_TEST_OPUS_VECTORS").map(PathBuf::from) else {
        support::available(false, "MIX_TEST_OPUS_VECTORS is unset");
        return;
    };
    if !support::available(Command::new(root.join("opus_compare")).output().is_ok(), "opus_compare is missing or cannot execute") { return; }
    for set in ["opus_testvectors", "opus_newvectors"] {
        for vector in 1..=12 {
            for extension in ["bit", "dec"] {
                if !support::available(root.join(set).join(format!("testvector{vector:02}.{extension}")).is_file(), "RFC vector fixture is missing") { return; }
            }
        }
    }
    let directory = support::TempDir::new("vectors");
    let work = directory.path();
    for set in ["opus_testvectors", "opus_newvectors"] {
        for channels in [Channels::Mono, Channels::Stereo] {
            for vector in 1..=12 {
                let stem = root.join(set).join(format!("testvector{vector:02}"));
                let bytes = fs::read(stem.with_extension("bit")).unwrap();
                let mut decoder = OpusDecoder::new(channels).unwrap();
                let mut at = 0;
                let mut pcm = Vec::new();
                let mut previous_lost = true;
                while at < bytes.len() {
                    let n = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
                    let expected_range = u32::from_be_bytes(bytes[at + 4..at + 8].try_into().unwrap());
                    at += 8; // payload length and encoder final range
                    let decoded = decoder.decode(Some(&bytes[at..at + n])).unwrap();
                    if expected_range != 0 && n != 0 && !previous_lost {
                        assert_eq!(decoder.final_range(), expected_range, "{set} {channels:?} {vector:02} packet offset {at}");
                    }
                    previous_lost = n == 0;
                    for v in decoded.pcm {
                        pcm.extend_from_slice(
                            &((v * 32768.).round_ties_even().clamp(-32768., 32767.) as i16)
                                .to_le_bytes(),
                        );
                    }
                    at += n;
                }
                let reference_bytes =
                    fs::metadata(stem.with_extension("dec")).unwrap().len() as usize;
                assert_eq!(pcm.len(), reference_bytes / 2 * channels.count());
                if set == "opus_newvectors" {
                    let out = work.join("decoded.pcm");
                    fs::write(&out, &pcm).unwrap();
                    let mut passed = false;
                    for suffix in ["", "m"] {
                        let mut command = Command::new(root.join("opus_compare"));
                        if channels == Channels::Stereo {
                            command.arg("-s");
                        }
                        let result = command
                            .args(["-r", "48000"])
                            .arg(stem.with_file_name(format!("testvector{vector:02}{suffix}.dec")))
                            .arg(&out)
                            .output()
                            .expect("external opus_compare must be installed by setup");
                        println!(
                            "{set} {channels:?} {vector:02}{suffix}: {}",
                            String::from_utf8_lossy(&result.stderr)
                        );
                        passed |= result.status.success();
                    }
                    assert!(passed, "RFC 8251 vector {vector:02}, {channels:?}");
                }
                // RFC 8251 permits passing its updated set instead of the old
                // references. Legacy bitstreams still receive a full decode
                // and length check above; old quality misses are not concealed.
            }
        }
    }
}
