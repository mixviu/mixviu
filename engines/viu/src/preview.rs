// SPDX-License-Identifier: MIT OR Apache-2.0
//! A passive scene specimen; no behaviours or device actions are executed.
use crate::{
    command::SceneCommand,
    store::SceneStore,
    view::{Content, SceneMessage, SceneUi},
};
use serde_json::json;
use std::{
    ffi::OsString,
    path::Path,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

struct Tools {
    mix: PathBuf,
    validator: PathBuf,
    share: PathBuf,
}
impl Tools {
    fn for_binary(binary: &Path, explicit: Option<OsString>) -> Result<Self, String> {
        let prefix = binary
            .parent()
            .and_then(Path::parent)
            .ok_or("viu executable has no prefix")?;
        Ok(Self {
            mix: prefix.join("bin/mix"),
            share: prefix.join("share"),
            validator: explicit
                .map(PathBuf::from)
                .unwrap_or_else(|| prefix.join("share/viud/scripts/scene_preview.mix")),
        })
    }
}

struct Preview {
    tools: Tools,
    dir: PathBuf,
    model: Option<PathBuf>,
    fingerprint: Vec<Option<Vec<u8>>>,
    next: Instant,
    store: SceneStore,
    view: Option<SceneUi>,
    error: Option<String>,
}
impl Preview {
    fn refresh(&mut self, ctx: &egui::Context) {
        let files = [
            Some(self.dir.join("scene.mix")),
            Some(self.dir.join("template.conf.mix")),
            Some(self.dir.join("behaviour.mix")),
            self.model.clone(),
        ];
        let fingerprint: Vec<_> = files
            .iter()
            .map(|p| {
                p.as_ref().and_then(|p| {
                    (std::fs::metadata(p).ok()?.len() <= 262144)
                        .then(|| std::fs::read(p).ok())
                        .flatten()
                })
            })
            .collect();
        if fingerprint == self.fingerprint {
            return;
        }
        self.fingerprint = fingerprint;
        let result = (|| -> Result<_, String> {
            let mut command = std::process::Command::new(&self.tools.mix);
            command.env("MIX_SHARE", &self.tools.share);
            command.arg(&self.tools.validator).arg(&self.dir);
            if let Some(model) = &self.model {
                command.arg(model);
            }
            let output = command.output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).into_owned());
            }
            let source = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
            self.store
                .request_mounted(SceneCommand::Load, &source, &json!({}), None)
                .map_err(|e| e.to_string())?;
            let (_, entry) = self.store.scenes().next().ok_or("missing scene")?;
            Ok(Arc::new(Content {
                tree: entry.tree().clone(),
                lists: entry.prepared().clone(),
                revision: entry.revision(),
                frame: None,
                dialog: false,
            }))
        })();
        match result {
            Ok(content) => {
                if let Some(view) = &mut self.view {
                    view.update(SceneMessage::Replace(content));
                } else {
                    let mut view = SceneUi::new(content);
                    view.setup(ctx);
                    self.view = Some(view);
                }
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }
}
impl eframe::App for Preview {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if Instant::now() >= self.next {
            self.refresh(&ctx);
            self.next = Instant::now() + Duration::from_millis(200);
        }
        ctx.request_repaint_after(Duration::from_millis(200));
        if let Some(view) = &mut self.view {
            view.show(ui, &mut Vec::new());
        }
        if let Some(error) = &self.error {
            crate::dev_loop::error_strip(ui, error);
        }
    }
}
pub fn run(args: &[String]) -> eframe::Result {
    let valid = args.len() == 1 || (args.len() == 3 && args[1] == "--model");
    if !valid {
        eprintln!("usage: viu --preview DIR [--model FILE]");
        std::process::exit(2);
    }
    let dir = PathBuf::from(&args[0]);
    let model = args.get(2).map(PathBuf::from);
    let tools = std::env::current_exe()
        .map_err(|e| e.to_string())
        .and_then(|exe| Tools::for_binary(&exe, std::env::var_os("VIU_PREVIEW_VALIDATOR")))
        .unwrap_or_else(|error| {
            eprintln!("viu preview: {error}");
            std::process::exit(1);
        });
    eframe::run_native(
        "Scene Preview",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Ok(Box::new(Preview {
                tools,
                dir,
                model,
                fingerprint: Vec::new(),
                next: Instant::now(),
                store: SceneStore::default(),
                view: None,
                error: None,
            }))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn planted_ancestor_validator_is_not_executed() {
        let temp = tempfile::tempdir().unwrap();
        let prefix = temp.path().join("installed");
        let ancestor = temp.path().join("download");
        let cwd = ancestor.join("nested/scene");
        let planted = ancestor.join("engines/viud/scripts");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(&planted).unwrap();
        std::fs::create_dir_all(prefix.join("bin")).unwrap();
        std::fs::create_dir_all(prefix.join("share/viud/scripts")).unwrap();
        let marker = temp.path().join("executed");
        std::fs::write(
            planted.join("scene_preview.mix"),
            format!("write_file({}, \"bad\")", json!(marker)),
        )
        .unwrap();
        std::fs::write(
            prefix.join("share/viud/scripts/scene_preview.mix"),
            "print(\"trusted\")",
        )
        .unwrap();
        std::os::unix::fs::symlink(crate::dev_loop::tests::test_mix(), prefix.join("bin/mix"))
            .unwrap();
        let tools = Tools::for_binary(&prefix.join("bin/viu"), None).unwrap();
        let output = std::process::Command::new(tools.mix)
            .arg(tools.validator)
            .current_dir(&cwd)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "trusted");
        assert!(!marker.exists());
        assert_eq!(
            Tools::for_binary(
                &prefix.join("bin/viu"),
                Some(OsString::from("explicit.mix"))
            )
            .unwrap()
            .validator,
            PathBuf::from("explicit.mix")
        );
    }
    #[test]
    fn preview_loads_recorded_model_and_retains_last_good_after_bad_edit() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("preview-test");
        let lib = temp.path().join("lib");
        std::fs::create_dir(&lib).unwrap();
        std::fs::write(
            lib.join("helper.mix"),
            "print(\"must not execute or enter transport\")",
        )
        .unwrap();
        std::fs::create_dir(&dir).unwrap();
        let source = "---\nscene: 1\nname: preview-test\ncitizen: preview-test\nmodel: {\"text\":\"stub\"}\n---\n```mix\nroot: {widget: text, text: '= $model.text'}\n```\n";
        std::fs::write(dir.join("scene.mix"), source).unwrap();
        std::fs::write(dir.join("template.conf.mix"), "{title: Preview}").unwrap();
        let model = temp.path().join("model.json");
        std::fs::write(&model, r#"{"text":"recorded"}"#).unwrap();
        let mut preview = Preview {
            tools: Tools {
                mix: crate::dev_loop::tests::test_mix(),
                validator: PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../viud/scripts/scene_preview.mix"
                )),
                share: PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../share")),
            },
            dir: dir.clone(),
            model: Some(model),
            fingerprint: Vec::new(),
            next: Instant::now(),
            store: SceneStore::default(),
            view: None,
            error: None,
        };
        let ctx = egui::Context::default();
        preview.refresh(&ctx);
        assert!(preview.error.is_none(), "{:?}", preview.error);
        assert_eq!(
            preview.store.scene("preview-test").unwrap().tree().nodes["root"].ports["text"],
            "recorded"
        );
        let revision = preview.store.scene("preview-test").unwrap().revision();
        std::fs::write(dir.join("scene.mix"), "broken").unwrap();
        preview.refresh(&ctx);
        assert!(preview.error.is_some() && preview.view.is_some());
        assert_eq!(
            preview.store.scene("preview-test").unwrap().revision(),
            revision
        );
        std::fs::write(dir.join("scene.mix"), source).unwrap();
        preview.refresh(&ctx);
        assert!(preview.error.is_none());
        std::fs::write(lib.join("helper.mix"), "$broken = [").unwrap();
        std::fs::write(dir.join("scene.mix"), format!("{source}\n")).unwrap();
        preview.refresh(&ctx);
        assert!(
            preview.error.is_some() && preview.view.is_some(),
            "preview validates helper syntax"
        );
    }
}
