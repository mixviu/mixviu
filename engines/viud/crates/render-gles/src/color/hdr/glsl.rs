// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The HDR shaders, as smithay custom texture shaders (GLSL ES 1.00).
//!
//! Two stages, matching [`super::pass`]:
//!
//! - **decode**: a colour-managed surface's texels to the composite's blending
//!   space. That space is KWin's choice: gamma 2.2 encoded, BT.709 primaries,
//!   1.0 = the SDR reference white, extended range (above 1 for HDR
//!   highlights, below 0 for colours outside BT.709), held in an FP16 target.
//!   Untagged SDR content is already in it, so only tagged surfaces pay.
//! - **encode**: the composite to the wire: BT.2020 primaries, absolute
//!   luminance (reference white = the configured SDR white), PQ.
//!
//! Every constant and curve is a transcription of [`super::math`], which
//! carries the tests.

use smithay::backend::renderer::gles::{UniformName, UniformType};

/// The decode stage's uniforms (names and types), declared by every program
/// that runs [`DECODE_FN`]: the plain decode program and every texture
/// program's HDR variant ([`inject_decode`]).
pub fn decode_uniforms() -> Vec<UniformName<'static>> {
    vec![
        UniformName::new("u_tf", UniformType::_1i),
        UniformName::new("u_prim", UniformType::Matrix3x3),
        UniformName::new("u_lum_max", UniformType::_1f),
        UniformName::new("u_inv_ref", UniformType::_1f),
        UniformName::new("u_hlg_gamma", UniformType::_1f),
        UniformName::new("u_sdr_white", UniformType::_1f),
        UniformName::new("u_tm", UniformType::_3f),
    ]
}

/// The encode stage's uniforms.
pub fn encode_uniforms() -> Vec<UniformName<'static>> {
    vec![
        UniformName::new("u_to2020", UniformType::Matrix3x3),
        UniformName::new("u_sdr_white", UniformType::_1f),
        UniformName::new("u_out_max", UniformType::_1f),
    ]
}

/// Shared helpers: sign-preserving power and the PQ pair.
const COMMON: &str = r#"
vec3 viu_spow(vec3 x, float p) {
    return sign(x) * pow(abs(x), vec3(p));
}

const float VIU_M1 = 0.1593017578125;
const float VIU_M2 = 78.84375;
const float VIU_C1 = 0.8359375;
const float VIU_C2 = 18.8515625;
const float VIU_C3 = 18.6875;

vec3 viu_pq_eotf(vec3 e) {
    vec3 p = pow(clamp(e, 0.0, 1.0), vec3(1.0 / VIU_M2));
    return 10000.0 * pow(max(p - VIU_C1, 0.0) / (VIU_C2 - VIU_C3 * p), vec3(1.0 / VIU_M1));
}

vec3 viu_pq_inv(vec3 nits) {
    vec3 p = pow(clamp(nits / 10000.0, 0.0, 1.0), vec3(VIU_M1));
    return pow((VIU_C1 + VIU_C2 * p) / (1.0 + VIU_C3 * p), vec3(VIU_M2));
}
"#;

/// The decode function, `vec4 viu_decode(vec4 premultiplied)`.
const DECODE: &str = r#"
uniform int u_tf;
uniform mat3 u_prim;
uniform float u_lum_max;
uniform float u_inv_ref;
uniform float u_hlg_gamma;
uniform float u_sdr_white;
// x: tone map on (1) or off (0); y: the content peak and z: the display peak,
// both as PQ code values, in the composite's absolute output luminance.
uniform vec3 u_tm;

vec3 viu_hlg_inv(vec3 e) {
    e = clamp(e, 0.0, 1.0);
    vec3 lo = e * e / 3.0;
    vec3 hi = (exp((e - 0.55991073) / 0.17883277) + 0.28466892) / 12.0;
    return mix(lo, hi, step(vec3(0.5), e));
}

float viu_bt2390(float e, float src, float dst) {
    if (src <= dst || src <= 0.0) {
        return min(e, max(src, dst));
    }
    float e1 = clamp(e / src, 0.0, 1.0);
    float max_lum = dst / src;
    float ks = 1.5 * max_lum - 0.5;
    if (e1 < ks || ks >= 1.0) {
        return e1 * src;
    }
    float t = (e1 - ks) / (1.0 - ks);
    float t2 = t * t;
    float t3 = t2 * t;
    float e2 = (2.0 * t3 - 3.0 * t2 + 1.0) * ks
        + (t3 - 2.0 * t2 + t) * (1.0 - ks)
        + (-2.0 * t3 + 3.0 * t2) * max_lum;
    return e2 * src;
}

vec4 viu_decode(vec4 c) {
    if (c.a <= 0.0) {
        return c;
    }
    vec3 e = c.rgb / c.a;
    vec3 nits;
    if (u_tf == 1) {
        nits = viu_pq_eotf(e);
    } else if (u_tf == 2) {
        vec3 s = viu_hlg_inv(e);
        float y = max(dot(vec3(0.2627, 0.6780, 0.0593), s), 1e-6);
        nits = u_lum_max * pow(y, u_hlg_gamma - 1.0) * s;
    } else if (u_tf == 3) {
        nits = e * u_lum_max;
    } else if (u_tf == 4) {
        nits = viu_spow(e, 2.4) * u_lum_max;
    } else if (u_tf == 5) {
        nits = viu_spow(e, 2.8) * u_lum_max;
    } else {
        nits = viu_spow(e, 2.2) * u_lum_max;
    }
    vec3 rel = (u_prim * nits) * u_inv_ref;
    if (u_tm.x > 0.5) {
        float peak = max(max(rel.r, rel.g), rel.b) * u_sdr_white;
        if (peak > 0.0) {
            float e0 = viu_pq_inv(vec3(peak)).r;
            float mapped = viu_pq_eotf(vec3(viu_bt2390(e0, u_tm.y, u_tm.z))).r;
            rel *= mapped / peak;
        }
    }
    return vec4(viu_spow(rel, 1.0 / 2.2) * c.a, c.a);
}
"#;

/// The header every smithay texture shader carries.
const HEADER: &str = r#"#version 100

//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision highp float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif
"#;

const TAIL: &str = r#"
#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color = vec4(0.0, 0.2, 0.0, 0.2) + color * 0.8;
#endif

    gl_FragColor = color;
}
"#;

/// The decode stage as a whole texture program: smithay's default texture
/// shader with [`viu_decode`] applied to the sampled texel.
pub fn decode_shader() -> String {
    format!(
        "{HEADER}{COMMON}{DECODE}
void main() {{
    vec4 color = texture2D(tex, v_coords);
#if defined(NO_ALPHA)
    color = vec4(color.rgb, 1.0);
#endif
    color = viu_decode(color) * alpha;
{TAIL}"
    )
}

/// The encode stage: the FP16 composite (blending space) to BT.2020 PQ.
pub fn encode_shader() -> String {
    format!(
        "{HEADER}{COMMON}
uniform mat3 u_to2020;
uniform float u_sdr_white;
uniform float u_out_max;

void main() {{
    vec4 color = texture2D(tex, v_coords);
    vec3 rel = viu_spow(color.rgb, 2.2);
    vec3 nits = clamp(u_to2020 * rel, 0.0, 1.0e4) * u_sdr_white;
    nits = min(nits, vec3(u_out_max));
    color = vec4(viu_pq_inv(nits), 1.0) * alpha;
{TAIL}"
    )
}

/// The marker a texture program puts where its final colour is written, so its
/// HDR variant can decode it: `gl_FragColor = VIU_OUT(color);`. Programs
/// without it get no HDR variant (they draw undecoded, the SDR look).
pub const OUT_MARKER: &str = "VIU_OUT(";

/// A texture program's HDR variant: the decode helpers inserted before
/// `void main`, and `VIU_OUT(x)` defined as `viu_decode(x)`. Its SDR build
/// defines `VIU_OUT(x)` as `x` ([`sdr_variant`]). `None` when the source has
/// no `void main` or no [`OUT_MARKER`].
pub fn inject_decode(source: &str) -> Option<String> {
    if !source.contains(OUT_MARKER) {
        return None;
    }
    let at = source.find("void main")?;
    let (head, body) = source.split_at(at);
    Some(format!(
        "{head}{COMMON}{DECODE}\n#define VIU_OUT(x) viu_decode(x)\n{body}"
    ))
}

/// A texture program's SDR build: `VIU_OUT(x)` is the identity.
pub fn sdr_variant(source: &str) -> String {
    match source.find("void main") {
        Some(at) if source.contains(OUT_MARKER) => {
            let (head, body) = source.split_at(at);
            format!("{head}#define VIU_OUT(x) (x)\n{body}")
        }
        _ => source.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shaders_keep_smithays_defines_line_and_one_main() {
        for src in [decode_shader(), encode_shader()] {
            assert!(src.contains("\n//_DEFINES_\n"));
            assert_eq!(src.matches("void main").count(), 1);
            assert!(src.starts_with("#version 100"));
        }
    }

    #[test]
    fn injection_needs_the_marker_and_keeps_main_last() {
        let plain = "#version 100\nvoid main() { gl_FragColor = c; }";
        assert!(inject_decode(plain).is_none());
        assert_eq!(sdr_variant(plain), plain);
        let marked =
            "#version 100\nprecision highp float;\nvoid main() { gl_FragColor = VIU_OUT(c); }";
        let hdr = inject_decode(marked).unwrap();
        assert!(hdr.find("vec4 viu_decode").unwrap() < hdr.find("void main").unwrap());
        assert!(hdr.contains("#define VIU_OUT(x) viu_decode(x)"));
        assert!(sdr_variant(marked).contains("#define VIU_OUT(x) (x)"));
    }

    /// Every build smithay compiles (its `//_DEFINES_` variants), framed for
    /// an external validator: `cargo test -p render-gles dump_shaders --
    /// --ignored --nocapture`, then `glslangValidator` each block.
    pub fn variants(name: &str, src: &str) -> Vec<(String, String)> {
        [
            ("plain", ""),
            ("no_alpha", "#define NO_ALPHA\n"),
            ("external", "#define EXTERNAL\n"),
            ("debug", "#define DEBUG_FLAGS\n"),
        ]
        .iter()
        .map(|(v, d)| (format!("{name}-{v}"), src.replace("//_DEFINES_", d)))
        .collect()
    }

    pub fn dump(name: &str, src: &str) {
        for (n, s) in variants(name, src) {
            println!("=====SHADER {n}=====\n{s}\n=====END=====");
        }
    }

    #[test]
    #[ignore]
    fn dump_shaders() {
        dump("decode", &decode_shader());
        dump("encode", &encode_shader());
        // decor's corner clip, both builds (the seam compiles both).
        let file = include_str!("../../../../decor/src/clip.rs");
        let start = file.find("pub const SHADER: &str = r#\"").unwrap() + 28;
        let clip = &file[start..start + file[start..].find("\"#;").unwrap()];
        dump("clip-sdr", &sdr_variant(clip));
        dump("clip-hdr", &inject_decode(clip).unwrap());
    }

    #[test]
    fn the_glsl_constants_are_the_rust_ones() {
        // Transcription check: the PQ constants in COMMON equal math.rs's.
        assert!(COMMON.contains(&format!("{}", 2610.0 / 16384.0)));
        assert!(COMMON.contains(&format!("{}", 2523.0 / 4096.0 * 128.0)));
        assert!(COMMON.contains(&format!("{}", 3424.0 / 4096.0)));
        assert!(COMMON.contains(&format!("{}", 2413.0 / 4096.0 * 32.0)));
        assert!(COMMON.contains(&format!("{}", 2392.0 / 4096.0 * 32.0)));
    }
}
