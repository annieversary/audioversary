use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::GainParams;

#[derive(Lens)]
struct Data {
    params: Arc<GainParams>,
    peak_meter_l: Arc<AtomicF32>,
    peak_meter_r: Arc<AtomicF32>,
    lufs_meter: Arc<AtomicF32>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::from_size(400, 220)
}

pub(crate) fn create(
    params: Arc<GainParams>,
    editor_state: Arc<ViziaState>,
    peak_meter_l: Arc<AtomicF32>,
    peak_meter_r: Arc<AtomicF32>,
    lufs_meter: Arc<AtomicF32>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        cx.add_theme(include_str!("editor.css"));

        Data {
            params: params.clone(),
            peak_meter_l: peak_meter_l.clone(),
            peak_meter_r: peak_meter_r.clone(),
            lufs_meter: lufs_meter.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "vst-mommy")
                .font(assets::NOTO_SANS_THIN)
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            // peak

            // TODO remove labels from this one
            // Label::new(cx, "Peak").bottom(Pixels(-1.0));
            // PeakMeter::new(
            //     cx,
            //     Data::peak_meter_l.map(|meter| util::gain_to_db(meter.load(Ordering::Relaxed))),
            //     Some(Duration::from_millis(600)),
            // )
            // .top(Pixels(10.0));
            // PeakMeter::new(
            //     cx,
            //     Data::peak_meter_r.map(|meter| util::gain_to_db(meter.load(Ordering::Relaxed))),
            //     Some(Duration::from_millis(600)),
            // );

            // lufs

            Label::new(cx, "LUFS").bottom(Pixels(-1.0));
            PeakMeter::new(
                cx,
                Data::lufs_meter.map(|meter| meter.load(Ordering::Relaxed)),
                Some(Duration::from_millis(600)),
            )
            .top(Pixels(10.0));
            Label::new(
                cx,
                Data::lufs_meter.map(|m| {
                    let v = m.load(Ordering::Relaxed);

                    if v < -7.0 {
                        "mommy knows her little girl can do better~"
                    } else {
                        "good girl~"
                    }
                }),
            )
            .top(Pixels(2.0));

            // Label::new(cx, "by annieversary").bottom(Pixels(-1.0));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}
