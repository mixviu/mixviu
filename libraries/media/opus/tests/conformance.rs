//! Fixtures: RFC 6716 / RFC 8251, IETF Trust BSD terms. The worker setup checks
//! archive SHA256 before extraction; no vector data is committed here.
use media_opus::{Channels, Decoder, OpusDecoder};
use std::{fs, path::PathBuf, process::Command};

#[test]
fn rfc_vectors() {
    let root = PathBuf::from(
        std::env::var("OPUS_VECTOR_ROOT")
            .expect("run tools/opus_test_setup.mix on the worker before this test"),
    );
    let work = std::env::temp_dir().join(format!("media-opus-vectors-{}", std::process::id()));
    fs::create_dir(&work).unwrap();
    for set in ["opus_testvectors", "opus_newvectors"] {
        for channels in [Channels::Mono, Channels::Stereo] {
            for vector in 1..=12 {
                let stem = root.join(set).join(format!("testvector{vector:02}"));
                let bytes = fs::read(stem.with_extension("bit")).unwrap();
                let mut decoder = OpusDecoder::new(channels).unwrap();
                let mut at = 0;
                let mut pcm = Vec::new();
                while at < bytes.len() {
                    let n = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
                    at += 8; // payload length and encoder final range
                    let decoded = decoder.decode(Some(&bytes[at..at + n])).unwrap();
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
    fs::remove_dir_all(work).unwrap();
}
