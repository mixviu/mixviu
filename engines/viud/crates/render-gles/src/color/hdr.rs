// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! HDR output for the GLES composite: PQ / BT.2020 signalling is in the native
//! backend; this is the colour side. [`describe`] holds what surfaces and
//! outputs mean, [`math`] the curves (and their tests), [`glsl`] the shaders
//! and [`pass`] the two-stage composite (FP16 blending space, then PQ encode).

pub mod describe;
pub mod glsl;
pub mod math;
pub mod pass;
