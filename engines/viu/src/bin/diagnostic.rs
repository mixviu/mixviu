// SPDX-License-Identifier: MIT OR Apache-2.0
//! Same-widget controller diagnostic; excludes GPU and physical presentation.
use eframe::egui;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::{
    runtime::{Program, Runtime},
    view::SceneView,
};

#[cfg(test)]
#[test]
fn populated_diagnostic_preserves_equal_outputs_and_geometry() {
    main();
}

const BEHAVIOUR: &str = r#"
$ctx = viu_context()
$rows = []
for $i = 0 to 511
  push($rows, {id: "row_" .. to_string($i), text: "Row " .. to_string($i), cells: []})
end
$model = {rows: $rows, selected: "row_0", offset: 0}
fn publish_model()
  $model.visible = slice($model.rows, 0, 256)
  send "viu-local" "scenes.model" body=json_encode({generation:$ctx.generation,value:$model})
end
publish_model()
on intent
  $args = $event.args
  if $args.kind == "select" then
    $model.selected = $args.id
  elif $args.kind == "scroll" then
    $model.offset = $args.offset
  else
    for $i = 0 to $args.rows - 1
      $model.rows[$i].text = "Edit " .. to_string($args.revision) .. ":" .. to_string($i)
    end
  end
  publish_model()
  reply(0, "{}")
end
"#;
const SCENE: &str = "---\nscene: 1\nname: diagnostic\ncitizen: diagnostic\nmodel: {\"rows\":[],\"visible\":[],\"selected\":\"row_0\",\"offset\":0}\n---\n```mix\nroot: {widget:\"column\",children:[\"selected\",\"list\"]}\nselected:{widget:\"text\",text:\"= $model.selected\"}\nlist:{widget:\"list\",rows:\"= $model.visible\",row:\"row\",row_height:24}\nrow:{widget:\"row\",children:[\"text\"]}\ntext:{widget:\"text\",text:\"= $item.text\"}\n```\n";
fn native(model: &mut Value, args: &Value) {
    match args["kind"].as_str().unwrap() {
        "select" => model["selected"] = args["id"].clone(),
        "scroll" => model["offset"] = args["offset"].clone(),
        _ => {
            for i in 0..args["rows"].as_u64().unwrap() as usize {
                model["rows"][i]["text"] = json!(format!("Edit {}:{i}", args["revision"]));
            }
        }
    }
    model["visible"] = json!(model["rows"].as_array().unwrap()[..256].to_vec());
}
fn draw(ctx: &egui::Context, view: &mut SceneView, model: Value) {
    view.apply_model(model).unwrap();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    );
    output.textures_delta.clear();
}
fn summary(mut values: Vec<u128>) -> Value {
    values.sort_unstable();
    json!({"median_us":values[values.len()/2] as f64/1000.0,"p95_us":values[(values.len()-1)*95/100] as f64/1000.0,"p99_us":values[(values.len()-1)*99/100] as f64/1000.0,"raw_ns":values})
}
fn next(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> viu::runtime::Completion {
    wakes.recv_timeout(Duration::from_secs(20)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    done
}
fn main() {
    let profile = std::env::var_os("VIU_DIAGNOSTIC_PROFILE").is_some();
    let string_body = std::env::var_os("VIU_DIAGNOSTIC_STRING_BODY").is_some();
    // Attribution run only: timer/log overhead is not baseline throughput.
    let source = if profile {
        BEHAVIOUR.replace(r#"  $model.visible = slice($model.rows, 0, 256)
  send "viu-local" "scenes.model" body=json_encode({generation:$ctx.generation,value:$model})"#, r#"  $p0 = monotonic()
  $model.visible = slice($model.rows, 0, 256)
  $p1 = monotonic()
  $encoded = json_encode({generation:$ctx.generation,value:$model})
  $p2 = monotonic()
  send "viu-local" "scenes.model" body=$encoded
  $p3 = monotonic()
  eprint(json_encode({slice_us:($p1-$p0)*1000000,prepare_us:($p2-$p1)*1000000,send_us:($p3-$p2)*1000000}))"#)
    } else { BEHAVIOUR.into() };
    let source = if string_body { source } else {
        source.replace("json_encode({generation:$ctx.generation,value:$model})", "{generation:$ctx.generation,value:$model}")
    };
    let started = Instant::now();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(Program {
            path: PathBuf::from("/tmp/viu-diagnostic.mix"),
            source,
            scene: "diagnostic".into(),
            citizen: "diagnostic".into(),
        })
        .unwrap();
    let mut expected = next(&runtime, &rx).model.unwrap();
    let startup_us = started.elapsed().as_micros();
    let ctx_mix = egui::Context::default();
    let ctx_rust = egui::Context::default();
    let mut mix_view = SceneView::new(SCENE).unwrap();
    let mut rust_view = SceneView::new(SCENE).unwrap();
    let mut results = Vec::new();
    for workload in ["select", "scroll", "edit_1", "edit_256", "edit_512"] {
        let mut mix_wall = Vec::new();
        let mut rust_wall = Vec::new();
        let mut service = Vec::new();
        let mut stages = Vec::new();
        for iteration in 0..35 {
            let args = match workload {
                "select" => json!({"kind":"select","id":format!("row_{}",iteration*7)}),
                "scroll" => json!({"kind":"scroll","offset":iteration*24}),
                _ => {
                    json!({"kind":"edit","rows":workload.strip_prefix("edit_").unwrap().parse::<usize>().unwrap(),"revision":iteration})
                }
            };
            let mut rust_ns = 0;
            if iteration % 2 == 0 {
                let start = Instant::now();
                native(&mut expected, &args);
                draw(&ctx_rust, &mut rust_view, expected.clone());
                rust_ns = start.elapsed().as_nanos();
            }
            let start = Instant::now();
            runtime.event("intent".into(), args.clone()).unwrap();
            let done = next(&runtime, &rx);
            if profile && iteration >= 4 {
                stages.push(serde_json::from_str::<Value>(done.stderr.trim()).expect("profile stage receipt"));
            }
            let actual = done.model.unwrap();
            draw(&ctx_mix, &mut mix_view, actual.clone());
            let mix_ns = start.elapsed().as_nanos();
            if iteration % 2 != 0 {
                let start = Instant::now();
                native(&mut expected, &args);
                draw(&ctx_rust, &mut rust_view, expected.clone());
                rust_ns = start.elapsed().as_nanos();
            }
            assert_eq!(
                actual, expected,
                "equal controller output before timing is accepted"
            );
            assert_eq!(
                mix_view.geometry, rust_view.geometry,
                "same widget geometry"
            );
            assert!(
                mix_view.geometry.len() > 256,
                "the list must actually draw populated row instances"
            );
            if iteration >= 4 {
                mix_wall.push(mix_ns);
                rust_wall.push(rust_ns);
                service.push(done.elapsed.as_nanos());
            }
        }
        results.push(json!({"workload":if workload=="scroll" {"scroll intent/model field update (no widget scrolling)"} else {workload},"mix_wall":summary(mix_wall),"rust_wall":summary(rust_wall),"mix_worker_service":summary(service),"profile_stages_us":stages}));
    }
    let idle = Instant::now();
    assert!(rx.recv_timeout(Duration::from_millis(100)).is_err());
    println!(
        "{}",
        json!({"profile_instrumented":profile,"publication":if string_body {"legacy JSON string"} else {"typed native handler"},"boundary":"synthetic512-row controller with256 visible rows + JSON model + same egui scene binding/template/paint CPU; excludes GPU, broker, compositor and physical presentation","samples":31,"warmups":4,"startup_model_ready_us":startup_us,"idle_no_completion_ms":idle.elapsed().as_millis(),"resources":std::fs::read_to_string("/proc/self/status").unwrap_or_default(),"results":results})
    );
}
