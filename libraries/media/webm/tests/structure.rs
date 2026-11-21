use media_webm::{Demuxer, Error, Video, Writer, av1_config};
use std::io::Cursor;

fn writer()->Writer<Cursor<Vec<u8>>> {
    Writer::new(Cursor::new(Vec::new()),Video{width:16,height:16,codec_private:vec![0x81,0,12,0]},None).unwrap()
}
#[test]
fn cluster_boundaries_and_cue_seek() {
    let mut w=writer();
    for (t,key) in [(0,true),(4999,false),(5000,false),(6000,true),(11000,false),(11001,true)] {
        w.packet(1,t,1,key,&[42]).unwrap();
    }
    let b=w.finish().unwrap().into_inner();
    assert_eq!(b.windows(4).filter(|x|*x==[0x1f,0x43,0xb6,0x75]).count(),5);
    let mut d=Demuxer::new(&b).unwrap(); assert_eq!(d.cues.len(),3);
    for (t,want) in [(0,0),(5999,0),(6000,6000),(11000,6000),(11001,11001),(u64::MAX,11001)] {
        assert_eq!(d.seek(t),Some(want)); let p=d.next_packet().unwrap();assert_eq!(p.timestamp,want as i64);assert!(p.key_frame);
    }
}
#[test]
fn unknown_ids_and_size_errors() {
    let mut w=writer();w.packet(1,0,1,true,&[42]).unwrap();let b=w.finish().unwrap().into_inner();
    // The writer's reserved Void is an unknown ID to the reader and is skipped.
    assert!(Demuxer::new(&b).is_ok());
    let mut hostile=b.clone();
    // EBML header size with all payload bits set except the last bit: huge finite size.
    hostile[4..12].copy_from_slice(&[1,255,255,255,255,255,255,254]);
    assert!(matches!(Demuxer::new(&hostile),Err(Error::Truncated)|Err(Error::SizeOverflow)));
    let mut hostile=b.clone();hostile[4]=0;
    assert!(matches!(Demuxer::new(&hostile),Err(Error::SizeOverflow)));
    // Unknown-sized Segment is the supported streaming-input fallback.
    let mut streaming=b.clone();let seg=streaming.windows(4).position(|b|b==[0x18,0x53,0x80,0x67]).unwrap();
    streaming[seg+4..seg+12].copy_from_slice(&[1,255,255,255,255,255,255,255]);
    assert!(Demuxer::new(&streaming).is_ok());
}
#[test]
fn invalid_calls_and_sequence_headers() {
    let mut w=writer();w.packet(1,100,1,true,&[42]).unwrap();assert!(w.packet(1,99,1,false,&[42]).is_err());assert!(w.finish().is_err());
    let mut w=writer();assert!(w.packet(2,0,1,true,&[42]).is_err());
    let mut w=writer();assert!(w.packet(1,u64::MAX,1,true,&[42]).is_err());
    for b in [&[][..],&[0x0a],&[0x0a,0xff],&[0x0a,1,0],&[0xff]] {assert!(av1_config(b).is_err());}
}
