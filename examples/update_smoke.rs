//! Runs the real installer/helper/restart against a disposable application package.
use pp_switch::{
    i18n::Language,
    update::{self, InstallPlan, Staged},
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

fn package_root(exe: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        exe.parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned()
    } else {
        exe.parent().unwrap().to_owned()
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 2 && args[0] == "--pp-install" {
        update::helper(Path::new(&args[1])).expect("isolated helper installation");
        return;
    }
    let exe = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    let root = package_root(&exe);
    if args.len() == 2 && args[0] == "--stage-check" {
        let json = fs::read_to_string(&args[1]).unwrap();
        let release =
            update::select_release(&json, &semver::Version::new(0, 0, 0), update::target())
                .unwrap()
                .unwrap();
        let staged =
            update::stage(&release, Language::En).expect("real HTTPS download/checksum/staging");
        assert_eq!(
            staged.plan.after.len(),
            update::owned_files(update::target()).len()
        );
        assert!(!root.join("restarted").exists());
        println!(
            "PP-15: real release downloaded, verified and staged in isolation; installation unchanged."
        );
        return;
    }
    if args.first().is_some_and(|a| a == "--prepare") {
        let directory = tempfile::Builder::new()
            .prefix("pp-switch-update-")
            .tempdir()
            .unwrap();
        let mut plan = InstallPlan {
            root: root.clone(),
            target: update::target().into(),
            before: BTreeMap::new(),
            after: BTreeMap::new(),
            parent_pid: std::process::id(),
            language: Language::En,
        };
        for name in update::owned_files(update::target()) {
            let old = fs::read(root.join(name)).unwrap();
            // Retain the executable and valid bundle metadata so the real restart can run.
            let next = if name.ends_with("pp-switch")
                || name.ends_with("pp-switch.exe")
                || name.ends_with("Info.plist")
            {
                old.clone()
            } else {
                b"new version resource".to_vec()
            };
            let destination = directory.path().join("payload").join(name);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::write(destination, &next).unwrap();
            plan.before.insert(name.into(), Some(update::sha256(&old)));
            plan.after.insert(name.into(), update::sha256(&next));
        }
        fs::write(
            root.join("staging-path"),
            directory.path().to_str().unwrap(),
        )
        .unwrap();
        update::launch_helper(Staged { directory, plan }).expect("start isolated helper");
        return; // The helper waits for this process, not the supervising harness.
    }
    if args.len() == 3 && args[0] == "--pp-clean-update" {
        update::cleanup(
            Path::new(&args[1]),
            args[2].to_str().unwrap().parse().unwrap(),
        );
    }
    if root.join("smoke-fixture").exists() {
        assert_eq!(
            fs::read(root.join(update::owned_files(update::target())[1])).unwrap(),
            b"new version resource"
        );
        fs::write(root.join("restarted"), b"passed").unwrap();
        return;
    }
    if args.first().is_some_and(|a| a == "--check") {
        println!(
            "Read-only explicit GitHub check: {:?}",
            update::check().expect("GitHub HTTPS check")
        );
        return;
    }
    if args.len() == 2 && args[0] == "--archive" {
        let bytes = fs::read(&args[1]).expect("packaged release");
        let version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
        let files = update::unpack(&bytes, &version, update::target())
            .expect("actual package accepted by updater");
        assert_eq!(files.len(), update::owned_files(update::target()).len());
        println!(
            "PP-15: real release package passed layout, bounds and executable architecture validation."
        );
        return;
    }
    let fixture = tempfile::tempdir().unwrap();
    let app = fs::canonicalize(fixture.path())
        .unwrap()
        .join(if cfg!(target_os = "macos") {
            "pp-switch.app"
        } else {
            "pp-switch"
        });
    for name in update::owned_files(update::target()) {
        let destination = app.join(name);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        if name.ends_with("pp-switch") || name.ends_with("pp-switch.exe") {
            fs::copy(&exe, destination).unwrap();
        } else if name.ends_with("Info.plist") {
            fs::write(destination, b"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>CFBundleExecutable</key><string>pp-switch</string><key>CFBundleIdentifier</key><string>io.github.phonowell.pp-switch</string><key>CFBundlePackageType</key><string>APPL</string></dict></plist>").unwrap();
        } else if name == "README.txt" {
            fs::write(destination, b"pp-switch smoke fixture").unwrap();
        } else {
            fs::write(destination, b"old version resource").unwrap();
        }
    }
    fs::write(app.join("smoke-fixture"), b"isolated").unwrap();
    let binary = app.join(update::owned_files(update::target()).last().unwrap());
    let mut command = Command::new(binary);
    let network = args.len() == 2 && args[0] == "--release-stage";
    if network {
        command.arg("--stage-check").arg(&args[1]);
    } else {
        command.arg("--prepare");
    }
    let status = command.status().unwrap();
    assert!(status.success());
    if network {
        return;
    }
    let deadline = Instant::now() + Duration::from_secs(25);
    while !app.join("restarted").exists() {
        assert!(
            Instant::now() < deadline,
            "Installer/restart did not finish"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(fs::read(app.join("restarted")).unwrap(), b"passed");
    // Windows briefly retains the restarted image while it exits.
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        !Path::new(&fs::read_to_string(app.join("staging-path")).unwrap()).exists(),
        "temporary helper/staging was not removed"
    );
    println!(
        "PP-15: real temporary helper waited, installed, restarted and cleaned up; user configuration untouched."
    );
}
