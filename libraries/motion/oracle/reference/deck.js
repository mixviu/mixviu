// MixViu episode deck runtime. Two modes, chosen by the URL hash:
//
//   #s=ID&b=N                    still mode: build step N of scene ID.
//     Elements carry data-b="N" (appear at step N and stay, keeping their
//     space) and optionally data-until="M" (shown only for steps N..M, out of
//     the layout otherwise).
//
//   #s=ID&t=SEC&beats=T0,T1,...  animated mode (build.mix, animate: true):
//     one video frame at time SEC within the scene; beat k starts at Tk
//     seconds. Every [data-b="k"] element's animation starts at Tk (+ its
//     data-delay); children of [data-stagger="S"] start S seconds apart;
//     data-until="M" hides an element from beat M+1 on; .count elements count
//     up to data-to from their beat. All CSS animations are then paused and
//     seeked to SEC, so a frame depends only on (scene, SEC, beats).
//
// In both modes [data-f] chips get the "found" ring from the section's
// data-found-step (default 2), and the bottom rail is data-driven:
//   window.DECK = {brand, rail: [{key, label}], covers: {scene: [keys]}, seq: [scenes]}
// Sections with class "norail" hide the rail.
(function () {
  const D = window.DECK || {rail: [], covers: {}, seq: [], brand: ""};
  const q = new URLSearchParams(location.hash.slice(1));
  const id = q.get("s") || (D.seq[0] || "brand");
  const sec = document.getElementById(id);
  if (!sec) { document.body.innerHTML = "<h1 style='color:red;padding:80px'>missing scene " + id + "</h1>"; return; }
  sec.classList.add("on");
  const animated = q.has("t");
  const fstep = +(sec.dataset.foundStep || 2);

  if (!animated) {
    const b = +(q.get("b") || 0);
    sec.querySelectorAll("[data-b]").forEach(e => {
      const from = +e.dataset.b, swap = e.dataset.until !== undefined, until = swap ? +e.dataset.until : 999;
      if (b < from || b > until) { if (swap) e.style.display = "none"; else e.classList.add("hide"); }
    });
    sec.querySelectorAll("[data-f]").forEach(e => { if (b >= fstep) e.classList.add("found"); });
  } else {
    const t = +q.get("t");
    const beats = (q.get("beats") || "0").split(",").map(Number);
    const at = k => beats[Math.min(k, beats.length - 1)] ?? 0;
    document.body.classList.add("animate");
    // stagger: give each child of a [data-stagger] container the parent's beat
    sec.querySelectorAll("[data-stagger]").forEach(p => {
      const step = +p.dataset.stagger, k = p.dataset.b ?? "0";
      [...p.children].forEach((c, i) => {
        if (c.dataset.b === undefined) c.dataset.b = k;
        c.dataset.delay = String((+(c.dataset.delay || 0)) + i * step);
        if (!c.dataset.anim && p.dataset.childAnim) c.dataset.anim = p.dataset.childAnim;
      });
      if (p.dataset.b !== undefined && !p.dataset.anim) p.dataset.anim = "fade";
    });
    sec.querySelectorAll("[data-b]").forEach(e => {
      const start = at(+e.dataset.b) + (+(e.dataset.delay || 0));
      e.style.animationDelay = start + "s";
      e.querySelectorAll(".underline-draw").forEach(u => u.style.animationDelay = (start + .3) + "s");
      // data-until past the last beat means "to the end": at() clamps to the
      // last beat, which would otherwise hide the element as soon as it starts
      const u = e.dataset.until !== undefined ? +e.dataset.until + 1 : NaN;
      if (u < beats.length && t >= at(u)) e.style.display = "none";
    });
    sec.querySelectorAll("[data-f]").forEach(e => { if (t >= at(fstep)) e.classList.add("found"); });
    sec.querySelectorAll(".count").forEach(e => {
      const k = +(e.closest("[data-b]")?.dataset.b ?? 0), s0 = at(k) + (+(e.dataset.delay || .2));
      const x = Math.min(1, Math.max(0, (t - s0) / 1.4)), ease = 1 - Math.pow(1 - x, 3);
      e.textContent = Math.round(+e.dataset.to * ease).toLocaleString("en-AU");
    });
    // freeze every animation at exactly t
    const seek = () => document.getAnimations().forEach(a => { a.pause(); a.currentTime = t * 1000; });
    seek();
    requestAnimationFrame(seek);
  }

  const rail = document.getElementById("rail");
  if (!rail) return;
  if (sec.classList.contains("norail")) { rail.style.display = "none"; return; }
  const here = D.covers[id] || [], idx = D.seq.indexOf(id), seen = new Set();
  D.seq.slice(0, Math.max(idx, 0)).forEach(s => (D.covers[s] || []).forEach(x => seen.add(x)));
  const last = idx >= 0 && D.seq.slice(idx + 1).every(s => !(D.covers[s] || []).length);
  rail.innerHTML = D.rail.map(r => `<span class="${here.includes(r.key) ? "cur" : (seen.has(r.key) || (last && !here.length) ? "done" : "")}">${r.label}</span>`).join("")
    + `<span class="brand">${D.brand || ""}</span>`;
})();
