// Embed the Windows icon using the installed SDK; no runtime or build dependency.
use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sdk = PathBuf::from(env::var_os("WindowsSdkDir").unwrap_or_else(|| {
        PathBuf::from(
            env::var_os("ProgramFiles(x86)").unwrap_or_else(|| "C:\\Program Files (x86)".into()),
        )
        .join("Windows Kits/10")
        .into_os_string()
    }));
    let mut compilers: Vec<_> = fs::read_dir(sdk.join("bin"))
        .expect("Windows SDK bin directory")
        .filter_map(|entry| entry.ok().map(|e| e.path().join("x64/rc.exe")))
        .filter(|p| p.is_file())
        .collect();
    compilers.sort();
    let compiler = compilers.last().expect("Windows SDK resource compiler");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("assets/icon.ico");
    let source = out.join("icon.rc");
    fs::write(
        &source,
        format!(
            "1 ICON \"{}\"\n",
            icon.to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    let resource = out.join("icon.res");
    assert!(
        Command::new(compiler)
            .arg("/nologo")
            .arg(format!("/fo{}", resource.display()))
            .arg(source)
            .status()
            .expect("run Windows resource compiler")
            .success()
    );
    println!("cargo:rustc-link-arg-bins={}", resource.display());
}
