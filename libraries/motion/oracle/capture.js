// SPDX-License-Identifier: MIT OR Apache-2.0
// Executed inside the headless browser by motion_oracle.mix. No Node/CDP.
(() => {
  const durations = {up:.7,down:.7,left:.7,right:.7,pop:.8,zoom:.9,fade:.9,
    wipe:.9,drop:.8,flip:.8,spin:1.1,glow:2.4,kb:14,float:4,draw:1.4,
    blink:3.2,bg:20,under:.9,shine:3};
  // deck.js has already applied its animation class, beat and delay rules.
  // Read the underline's actual pseudo-element, rather than substituting CSS.
  const output = {schema:1, browser:navigator.userAgent, width:100, height:100,
    samples_per_preset:21, presets:[]};
  for (const [name,duration] of Object.entries(durations)) {
    const target = name === 'bg' ? document.body : document.getElementById(name);
    const pseudo = name === 'under' ? '::after' : null;
    const samples = [];
    const animations = document.getAnimations();
    for (let i=0;i<=20;i++) {
      const t = duration*i/20;
      // Identical fixed-time seek to deck.js, including every animation.
      animations.forEach(a => { a.pause(); a.currentTime=t*1000; });
      const s = getComputedStyle(target,pseudo);
      const m = new DOMMatrixReadOnly(s.transform === 'none' ? undefined : s.transform);
      const v = {opacity:+s.opacity};
      if (['up','down','drop','float'].includes(name)) v.ty=m.m42;
      if (['left','right'].includes(name)) v.tx=m.m41;
      if (['pop','zoom','drop'].includes(name)) v.scale=m.m11;
      if (name==='kb') {
        v.scale=m.m11;
        // CSS scale() precedes percent translate(); undo scale for the
        // logical-pixel translation at the reference 100x100 size.
        v.tx=m.m41/m.m11; v.ty=m.m42/m.m22;
      }
      if (name==='spin') {
        v.scale=Math.hypot(m.m11,m.m12);
        let angle=Math.atan2(m.m12,m.m11)*180/Math.PI;
        if (angle>0.00001) angle-=360;
        v.rotate=angle;
      }
      if (name==='under') v.scale_x=m.m11;
      if (name==='blink') v.scale_y=m.m22;
      if (name==='zoom') v.blur=s.filter==='none'?0:parseFloat(s.filter.slice(5));
      if (name==='wipe') {
        const parts=s.clipPath.slice(6,-1).split(/\s+/).map(parseFloat);
        const [a,b=a,c=a,d=b]=parts;
        Object.assign(v,{clip_top:a/100,clip_right:b/100,clip_bottom:c/100,clip_left:d/100});
      }
      if (name==='flip') v.scalar=Math.acos(Math.max(-1,Math.min(1,m.m22)))*180/Math.PI;
      if (name==='draw') v.dash_offset=parseFloat(s.strokeDashoffset);
      if (name==='glow') {
        // Shadow blur radius is a linear 0..40 channel; colour and spread
        // are renderer concerns. Remove colour before reading pixel lengths.
        const lengths=s.boxShadow.replace(/rgba?\([^)]*\)/g,'').match(/-?[\d.]+px/g)||[];
        v.scalar=lengths.length>2?parseFloat(lengths[2])/40:0;
      }
      if (name==='bg') v.scalar=parseFloat(s.backgroundPosition)/18;
      if (name==='shine') v.scalar=parseFloat(s.backgroundPosition)/100;
      // Opacity is an underlying value for presets which do not animate it.
      if (!['up','down','left','right','pop','zoom','fade','drop','flip','spin'].includes(name)) delete v.opacity;
      samples.push({t,values:v,raw:{opacity:s.opacity,transform:s.transform,
        filter:s.filter,clip_path:s.clipPath,box_shadow:s.boxShadow,
        background_position:s.backgroundPosition,dash_offset:s.strokeDashoffset}});
    }
    output.presets.push({name,duration,samples});
  }
  const pre=document.createElement('pre');
  pre.id='motion-oracle-result';
  pre.textContent=JSON.stringify(output);
  document.body.append(pre);
})();
