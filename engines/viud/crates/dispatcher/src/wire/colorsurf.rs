// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use crate::wire::color::{
    ImageDescData, ParamsState, chromaticity, output_description, preferred, primaries_from,
    send_info, send_ready, set_pending_color, track_feedback, track_output, transfer_from, wenum,
};
use render_gles::color::hdr::describe::{Description, Transfer};
use render_gles::color::hdr::math::BT709;
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::{
    wp_color_management_output_v1::{Request as OutReq, WpColorManagementOutputV1},
    wp_color_management_surface_feedback_v1::{
        Request as FbReq, WpColorManagementSurfaceFeedbackV1,
    },
    wp_color_management_surface_v1::{Request as SurfReq, WpColorManagementSurfaceV1},
    wp_color_manager_v1::Request as MgrReq,
    wp_image_description_creator_icc_v1::{Request as IccReq, WpImageDescriptionCreatorIccV1},
    wp_image_description_creator_params_v1::{Request as PrReq, WpImageDescriptionCreatorParamsV1},
    wp_image_description_info_v1::WpImageDescriptionInfoV1,
    wp_image_description_v1::{Cause, Request as DsReq, WpImageDescriptionV1},
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{DataInit, Dispatch, Resource};
use std::sync::Mutex;

type DI<'a, W> = DataInit<'a, W>;

/// Windows scRGB: linear BT.709, 1.0 = 80 cd/m², the output's volume as the
/// target (the protocol's definition).
fn scrgb() -> Description {
    let mut d = Description::new(Transfer::Linear, BT709);
    d.lum_min = 0.0;
    // RGB 1.0 is 80 cd/m² (the linear scale); the reference white the content
    // is anchored to is the protocol's assumed 203 cd/m², not 80, or scRGB
    // would come out 2.54x too bright.
    d.lum_max = 80.0;
    d.lum_ref = 203.0;
    let target = preferred();
    d.mastering_min = target.mastering_min;
    // The range scRGB can carry far exceeds 80 cd/m² (values above 1.0). With
    // no HDR output to take the target from, assume a 1000 cd/m² master, so an
    // SDR output tone-maps highlights instead of clipping them all to white.
    d.mastering_max = target.mastering_max.or(Some(1000.0));
    d.mastering_primaries = target.mastering_primaries;
    d
}

pub fn dispatch_color_manager<W>(req: MgrReq, di: &mut DI<'_, W>)
where
    W: Dispatch<WpColorManagementOutputV1, String> + 'static,
    W: Dispatch<WpColorManagementSurfaceV1, WlSurface> + 'static,
    W: Dispatch<WpColorManagementSurfaceFeedbackV1, WlSurface> + 'static,
    W: Dispatch<WpImageDescriptionCreatorIccV1, ()> + 'static,
    W: Dispatch<WpImageDescriptionCreatorParamsV1, Mutex<ParamsState>> + 'static,
    W: Dispatch<WpImageDescriptionV1, ImageDescData> + 'static,
{
    trace!("color: manager request {:?}", req);
    match req {
        MgrReq::Destroy => {}
        MgrReq::GetOutput { id, output } => {
            let name = smithay::output::Output::from_resource(&output)
                .map(|o| o.name())
                .unwrap_or_default();
            let obj = di.init(id, name.clone());
            track_output(&name, &obj);
        }
        MgrReq::GetSurface { id, surface } => {
            di.init(id, surface);
        }
        MgrReq::GetSurfaceFeedback { id, surface } => {
            let fb = di.init(id, surface);
            track_feedback(&fb);
        }
        MgrReq::CreateIccCreator { obj } => {
            di.init(obj, ());
        }
        MgrReq::CreateParametricCreator { obj } => {
            di.init(obj, Mutex::new(ParamsState::default()));
        }
        MgrReq::CreateWindowsScrgb { image_description } => {
            send_ready(&di.init(image_description, ImageDescData(Some(scrgb()))));
        }
        MgrReq::GetImageDescription {
            image_description, ..
        } => {
            send_ready(&di.init(image_description, ImageDescData(Some(preferred()))));
        }
        _ => {}
    }
}

pub fn dispatch_color_output<W: Dispatch<WpImageDescriptionV1, ImageDescData> + 'static>(
    req: OutReq,
    output: &str,
    di: &mut DI<'_, W>,
) {
    if let OutReq::GetImageDescription { image_description } = req {
        let d = output_description(output);
        trace!("color: output {output} description -> {:?}", d.transfer);
        send_ready(&di.init(image_description, ImageDescData(Some(d))));
    }
}

/// Set and unset are double-buffered: they take effect on the surface's next
/// commit, with the buffer they describe (`color::on_commit`). Destroying the
/// object unsets the description, also at the next commit.
pub fn dispatch_color_surface(
    req: SurfReq,
    resource: &WpColorManagementSurfaceV1,
    surface: &WlSurface,
) {
    trace!("color: surface request {:?}", req);
    match req {
        SurfReq::SetImageDescription {
            image_description, ..
        } => match image_description.data::<ImageDescData>().and_then(|d| d.0) {
            Some(d) => set_pending_color(surface, Some(d)),
            // Only ready descriptions may be set; a failed one is the
            // protocol's image_description error.
            None => resource.post_error(
                smithay::reexports::wayland_protocols::wp::color_management::v1::server::wp_color_management_surface_v1::Error::ImageDescription,
                "image description is not ready (it failed)",
            ),
        },
        SurfReq::UnsetImageDescription | SurfReq::Destroy => {
            set_pending_color(surface, None);
        }
        _ => {}
    }
}

pub fn dispatch_color_feedback<W: Dispatch<WpImageDescriptionV1, ImageDescData> + 'static>(
    req: FbReq,
    di: &mut DI<'_, W>,
) {
    match req {
        FbReq::GetPreferred { image_description }
        | FbReq::GetPreferredParametric { image_description } => {
            trace!("color: preferred description -> {:?}", preferred().transfer);
            send_ready(&di.init(image_description, ImageDescData(Some(preferred()))));
        }
        _ => {}
    }
}

pub fn dispatch_color_params<W: Dispatch<WpImageDescriptionV1, ImageDescData> + 'static>(
    req: PrReq,
    data: &Mutex<ParamsState>,
    di: &mut DI<'_, W>,
) {
    trace!("color: params request {:?}", req);
    let mut p = data.lock().unwrap_or_else(|e| e.into_inner());
    match req {
        PrReq::SetTfNamed { tf } => match wenum(tf).and_then(transfer_from) {
            Some(t) => p.transfer = Some(t),
            None => p.unsupported = Some("transfer function not supported"),
        },
        PrReq::SetTfPower { .. } => p.unsupported = Some("power transfer functions not supported"),
        PrReq::SetPrimariesNamed { primaries } => match wenum(primaries).and_then(primaries_from) {
            Some(c) => p.primaries = Some(c),
            None => p.unsupported = Some("named primaries not supported"),
        },
        PrReq::SetPrimaries {
            r_x,
            r_y,
            g_x,
            g_y,
            b_x,
            b_y,
            w_x,
            w_y,
        } => match chromaticity(r_x, r_y, g_x, g_y, b_x, b_y, w_x, w_y) {
            Some(c) => p.primaries = Some(c),
            None => p.unsupported = Some("degenerate primaries"),
        },
        PrReq::SetLuminances {
            min_lum,
            max_lum,
            reference_lum,
        } => {
            p.luminances = Some((
                min_lum as f64 / 10000.0,
                max_lum as f64,
                reference_lum as f64,
            ));
        }
        PrReq::SetMasteringDisplayPrimaries {
            r_x,
            r_y,
            g_x,
            g_y,
            b_x,
            b_y,
            w_x,
            w_y,
        } => {
            p.mastering_primaries = chromaticity(r_x, r_y, g_x, g_y, b_x, b_y, w_x, w_y);
        }
        PrReq::SetMasteringLuminance { min_lum, max_lum } => {
            p.mastering_luminance = Some((min_lum as f64 / 10000.0, max_lum as f64));
        }
        PrReq::SetMaxCll { max_cll } => p.max_cll = Some(max_cll as f64),
        PrReq::SetMaxFall { max_fall } => p.max_fall = Some(max_fall as f64),
        PrReq::Create { image_description } => match p.build() {
            Ok(d) => send_ready(&di.init(image_description, ImageDescData(Some(d)))),
            Err(why) => {
                warn!("color: parametric create failed: {why}");
                di.init(image_description, ImageDescData(None))
                    .failed(Cause::Unsupported, why.into());
            }
        },
        _ => {}
    }
}

pub fn dispatch_color_icc<W: Dispatch<WpImageDescriptionV1, ImageDescData> + 'static>(
    req: IccReq,
    di: &mut DI<'_, W>,
) {
    if let IccReq::Create { image_description } = req {
        di.init(image_description, ImageDescData(None))
            .failed(Cause::Unsupported, "ICC profiles not supported".into());
    }
}

/// `get_information`: the info events now, and the info object back to the
/// caller, which owes it `done` after dispatch (a destructor event; sending it
/// here frees the object wayland-backend is about to attach data to).
pub fn dispatch_image_desc<W: Dispatch<WpImageDescriptionInfoV1, ()> + 'static>(
    req: DsReq,
    data: &ImageDescData,
    di: &mut DI<'_, W>,
) -> Option<WpImageDescriptionInfoV1> {
    if let DsReq::GetInformation { information } = req {
        let info: WpImageDescriptionInfoV1 = di.init(information, ());
        if let Some(d) = &data.0 {
            send_info(&info, d);
        }
        trace!(
            "color: get_information answered ({:?}); done deferred",
            data.0.map(|d| d.transfer)
        );
        return Some(info);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use render_gles::color::hdr::describe::OutputColor;
    use smithay::backend::renderer::gles::UniformValue;

    fn tm(us: &[smithay::backend::renderer::gles::Uniform<'static>]) -> (f32, f32, f32) {
        match &us.iter().find(|u| u.name == "u_tm").unwrap().value {
            UniformValue::_3f(a, b, c) => (*a, *b, *c),
            v => panic!("{v:?}"),
        }
    }

    /// scRGB: 1.0 is 80 cd/m² anchored to a 203 cd/m² reference white, and with
    /// no HDR output its highlights are tone-mapped on an SDR target rather
    /// than all clipped to white (Sol round 2).
    #[test]
    fn scrgb_scale_reference_and_sdr_highlights() {
        let d = scrgb();
        assert_eq!((d.lum_max, d.lum_ref), (80.0, 203.0));
        assert_eq!(d.mastering_max, Some(1000.0));
        let (on, src, dst) = tm(&OutputColor::sdr().decode_uniforms(&d));
        assert_eq!(on, 1.0);
        assert!(
            src > dst,
            "content peak must exceed SDR white: {src} vs {dst}"
        );
    }
}
