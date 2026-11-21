use std::fs;
fn align<'a>(r:&'a [f32],v:&'a [f32],lag:i32)->(&'a [f32],&'a [f32]) {let (a,b)=if lag<0 {((-lag) as usize*2,0)}else{(0,lag as usize*2)};let n=(r.len()-a).min(v.len()-b);(&r[a..a+n],&v[b..b+n])}
fn snr(r:&[f32],v:&[f32])->(f64,i32) {
 let mut best=f64::NEG_INFINITY;let mut lag=0;
 for shift in -960i32..=960 {let (r,v)=align(r,v,shift);let mut s=0.;let mut e=0.;for i in (0..r.len()/2).step_by(8) {for c in 0..2 {let j=i*2+c;s+=(r[j] as f64).powi(2);e+=((r[j]-v[j]) as f64).powi(2);}}let db=10.*(s/e.max(1e-30)).log10();if db>best {best=db;lag=shift;}}
 let (r,v)=align(r,v,lag);let s:f64=r.iter().map(|&x|(x as f64).powi(2)).sum();let e:f64=r.iter().zip(v).map(|(&x,&y)|((x-y) as f64).powi(2)).sum();(10.*(s/e.max(1e-30)).log10(),lag)
}
pub fn report(a:&[String]) {
 let source=super::pcm(&a[2]);let read=|p:&str|->Vec<f32>{fs::read(p).unwrap().chunks_exact(2).map(|b|i16::from_le_bytes(b.try_into().unwrap()) as f32/32768.).collect()};let x=read(&a[3]);let y=read(&a[4]);
 let (sx,lx)=snr(&source,&x);let(sy,ly)=snr(&source,&y);let(xy,lxy)=snr(&x,&y);let(x,y)=align(&x,&y,lxy);
 let mut sd=0.;let mut bins=0;
 for win in 0..100 {let off=win*4096;if off+512>=x.len(){break}for c in 0..2 {for k in 1..128 {let mut xr=0.;let mut xi=0.;let mut yr=0.;let mut yi=0.;for j in 0..256 {let phase=std::f64::consts::TAU*k as f64*j as f64/256.;let w=0.5-0.5*(std::f64::consts::TAU*j as f64/255.).cos();let xx=x[off+j*2+c] as f64*w;let yy=y[off+j*2+c] as f64*w;xr+=xx*phase.cos();xi+=xx*phase.sin();yr+=yy*phase.cos();yi+=yy*phase.sin();}let db=10.*((xr*xr+xi*xi+1e-10)/(yr*yr+yi*yi+1e-10)).log10();sd+=db*db;bins+=1;}}}
 println!("source_candidate_snr={sx:.4} lag={lx} source_libopus_snr={sy:.4} lag={ly} candidate_libopus_snr={xy:.4} lag={lxy} spectral_rms_db={:.4}",(sd/bins as f64).sqrt());
}
