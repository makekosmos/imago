use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=windows/app.manifest");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let manifest = manifest_dir.join("windows/app.manifest");
    let rc = out_dir.join("window_chrome.rc");
    let manifest = manifest.to_string_lossy().replace('\\', "\\\\");
    fs::write(&rc, format!("1 24 \"{manifest}\"\n")).expect("write resource script");
    embed_resource::compile(&rc, embed_resource::NONE)
        .manifest_required()
        .expect("embed host-owned Windows manifest");
}
