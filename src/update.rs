//! User-triggered GitHub updates. Network work is never started by app startup.
use crate::{
    Result,
    i18n::{self, Language},
    storage,
};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use tempfile::{NamedTempFile, TempDir};
use url::Url;

const REPOSITORY: &str = "phonowell/pp-switch";
const ARCHIVE_LIMIT: u64 = 10 * 1024 * 1024;
const INSTALLED_LIMIT: u64 = 5 * 1024 * 1024;

pub fn target() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(windows, target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else {
        "unsupported"
    }
}

#[derive(Clone, Debug)]
pub struct Update {
    pub version: Version,
    pub url: String,
    pub digest: String,
    pub size: u64,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn asset_name(version: &Version, target: &str) -> String {
    format!(
        "pp-switch-v{version}-{target}.{}",
        if target.contains("linux") {
            "tar.gz"
        } else {
            "zip"
        }
    )
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
    size: u64,
}

pub fn select_release(json: &str, current: &Version, target: &str) -> Result<Option<Update>> {
    let releases: Vec<Release> =
        serde_json::from_str(json).map_err(|_| "Invalid update metadata.")?;
    let mut candidate = None;
    for release in releases.into_iter().filter(|r| !r.draft) {
        let Ok(version) = Version::parse(release.tag_name.strip_prefix('v').unwrap_or("")) else {
            continue;
        };
        if version <= *current
            || candidate
                .as_ref()
                .is_some_and(|u: &Update| u.version >= version)
        {
            continue;
        }
        let name = asset_name(&version, target);
        let matches: Vec<_> = release
            .assets
            .into_iter()
            .filter(|a| a.name == name)
            .collect();
        if matches.is_empty() {
            continue;
        }
        if matches.len() != 1 {
            return Err("Invalid update metadata.".into());
        }
        let asset = matches.into_iter().next().unwrap();
        let expected = format!(
            "https://github.com/{REPOSITORY}/releases/download/{}/{name}",
            release.tag_name
        );
        let digest = asset
            .digest
            .and_then(|s| s.strip_prefix("sha256:").map(str::to_owned))
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or("Update has no valid SHA-256 digest.")?;
        if asset.browser_download_url != expected || asset.size == 0 || asset.size > ARCHIVE_LIMIT {
            return Err("Invalid update metadata.".into());
        }
        candidate = Some(Update {
            version,
            url: expected,
            digest: digest.to_ascii_lowercase(),
            size: asset.size,
        });
    }
    Ok(candidate)
}

fn response(url: &str, deadline: Instant) -> Result<ureq::Response> {
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(8))
        .timeout_write(Duration::from_secs(8))
        .build();
    let mut url = Url::parse(url).map_err(|_| "Invalid update URL.")?;
    for _ in 0..=5 {
        if deadline <= Instant::now() {
            return Err("Update request timed out. Try again.".into());
        }
        if url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || !matches!(
                url.host_str(),
                Some(
                    "api.github.com"
                        | "github.com"
                        | "release-assets.githubusercontent.com"
                        | "objects.githubusercontent.com"
                )
            )
        {
            return Err("Invalid update URL.".into());
        }
        let result = agent
            .get(url.as_str())
            .set(
                "User-Agent",
                concat!("pp-switch/", env!("CARGO_PKG_VERSION")),
            )
            .set("Accept", "application/vnd.github+json")
            .call();
        let response = match result {
            Ok(r) | Err(ureq::Error::Status(301 | 302 | 303 | 307 | 308, r)) => r,
            _ => return Err("Cannot contact GitHub. Check your connection or try later.".into()),
        };
        if response.status() == 200 {
            return Ok(response);
        }
        url = response
            .header("Location")
            .and_then(|location| url.join(location).ok())
            .ok_or("Invalid update URL.")?;
    }
    Err("Too many update redirects.".into())
}

fn bounded_read(mut reader: impl Read, limit: u64, deadline: Instant) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 16 * 1024];
    loop {
        if Instant::now() >= deadline {
            return Err("Update request timed out. Try again.".into());
        }
        let count = reader
            .read(&mut buffer)
            .map_err(|_| "Cannot read update data.")?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len() as u64 + count as u64 > limit {
            return Err("Update data exceeds the size limit.".into());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

pub fn check() -> Result<Option<Update>> {
    let deadline = Instant::now() + Duration::from_secs(45);
    let url = format!("https://api.github.com/repos/{REPOSITORY}/releases?per_page=10");
    let bytes = bounded_read(
        response(&url, deadline)?.into_reader(),
        512 * 1024,
        deadline,
    )?;
    let json = std::str::from_utf8(&bytes).map_err(|_| "Invalid update metadata.")?;
    select_release(
        json,
        &Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        target(),
    )
}

pub fn owned_files(target: &str) -> Vec<&'static str> {
    if target.contains("apple") {
        vec![
            "Contents/Info.plist",
            "Contents/Resources/LICENSE",
            "Contents/Resources/THIRD_PARTY_NOTICES.txt",
            "Contents/Resources/README.txt",
            "Contents/MacOS/pp-switch",
        ]
    } else {
        vec![
            "LICENSE",
            "THIRD_PARTY_NOTICES.txt",
            "README.txt",
            if target.contains("windows") {
                "pp-switch.exe"
            } else {
                "pp-switch"
            },
        ]
    }
}

fn archive_path(name: &str, version: &Version, target: &str) -> Result<String> {
    let path = Path::new(name);
    if name.contains('\\')
        || name.len() > 512
        || !path.components().all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("Unsafe update archive path.".into());
    }
    let relative = if target.contains("apple") {
        name.strip_prefix("pp-switch.app/")
    } else if target.contains("linux") {
        name.strip_prefix(&format!("pp-switch-v{version}-{target}/"))
    } else {
        Some(name)
    }
    .ok_or("Unsupported update package layout.")?;
    if !owned_files(target).contains(&relative) {
        return Err("Unsupported update package layout.".into());
    }
    Ok(relative.to_owned())
}

fn archive_directory(name: &str, version: &Version, target: &str) -> Result<()> {
    let name = name.trim_end_matches('/');
    if name.contains('\\')
        || name.len() > 512
        || !Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("Unsafe update archive path.".into());
    }
    let root = if target.contains("apple") {
        "pp-switch.app".to_owned()
    } else if target.contains("linux") {
        format!("pp-switch-v{version}-{target}")
    } else {
        String::new()
    };
    let prefix = format!("{name}/");
    if name == root
        || owned_files(target).iter().any(|p| {
            let full = if root.is_empty() {
                (*p).to_owned()
            } else {
                format!("{root}/{p}")
            };
            full.starts_with(&prefix)
        })
    {
        Ok(())
    } else {
        Err("Unsupported update package layout.".into())
    }
}

pub fn unpack(bytes: &[u8], version: &Version, target: &str) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    let mut total = 0;
    let mut add = |name: &str, reader: &mut dyn Read| -> Result<()> {
        let relative = archive_path(name, version, target)?;
        let data = bounded_read(
            reader,
            INSTALLED_LIMIT - total,
            Instant::now() + Duration::from_secs(15),
        )?;
        total += data.len() as u64;
        if files.insert(relative, data).is_some() {
            return Err("Duplicate file in update package.".into());
        }
        Ok(())
    };
    if bytes.len() as u64 > ARCHIVE_LIMIT {
        return Err("Update data exceeds the size limit.".into());
    }
    if target.contains("linux") {
        // Bound total decompression too, including directory/PAX metadata.
        let reader = flate2::read::GzDecoder::new(bytes).take(INSTALLED_LIMIT + 64 * 1024);
        let mut archive = tar::Archive::new(reader);
        for (index, entry) in archive
            .entries()
            .map_err(|_| "Invalid update archive.")?
            .enumerate()
        {
            if index >= 32 {
                return Err("Unsupported update package layout.".into());
            }
            let mut entry = entry.map_err(|_| "Invalid update archive.")?;
            let name = entry
                .path()
                .map_err(|_| "Invalid update archive.")?
                .to_string_lossy()
                .into_owned();
            if entry.header().entry_type().is_dir() {
                if entry.size() != 0 {
                    return Err("Unsupported update package layout.".into());
                }
                archive_directory(&name, version, target)?;
                continue;
            }
            if !entry.header().entry_type().is_file() {
                return Err("Update package contains a link or special file.".into());
            }
            add(&name, &mut entry)?;
        }
    } else {
        let mut archive =
            zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "Invalid update archive.")?;
        if archive.len() > 32 {
            return Err("Unsupported update package layout.".into());
        }
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|_| "Invalid update archive.")?;
            if entry.enclosed_name().is_none() || entry.name().contains('\\') {
                return Err("Unsafe update archive path.".into());
            }
            if entry
                .unix_mode()
                .is_some_and(|m| !matches!(m & 0o170000, 0 | 0o100000 | 0o040000))
            {
                return Err("Update package contains a link or special file.".into());
            }
            if entry.is_dir() {
                if entry.size() != 0 {
                    return Err("Unsupported update package layout.".into());
                }
                archive_directory(entry.name(), version, target)?;
                continue;
            }
            let name = entry.name().to_owned();
            add(&name, &mut entry)?;
        }
    }
    if files.len() != owned_files(target).len() {
        return Err("Update package is missing required files.".into());
    }
    let binary = files.get(*owned_files(target).last().unwrap()).unwrap();
    let valid = if target.contains("apple") {
        binary.get(..4) == Some(&[0xcf, 0xfa, 0xed, 0xfe])
            && binary.get(4..8).is_some_and(|b| {
                u32::from_le_bytes(b.try_into().unwrap())
                    == if target.starts_with("aarch64") {
                        0x0100_000c
                    } else {
                        0x0100_0007
                    }
            })
    } else if target.contains("windows") {
        binary.get(..2) == Some(b"MZ")
            && binary.get(0x3c..0x40).is_some_and(|b| {
                let offset = u32::from_le_bytes(b.try_into().unwrap()) as usize;
                binary.get(offset..offset.saturating_add(6))
                    == Some(&[b'P', b'E', 0, 0, 0x64, 0x86])
            })
    } else {
        binary.get(..6) == Some(&[0x7f, b'E', b'L', b'F', 2, 1])
            && binary.get(18..20) == Some(&[62, 0])
    };
    if !valid {
        return Err("Update executable does not match this platform.".into());
    }
    Ok(files)
}

fn file_hash(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Ok(meta)
            if meta.is_file()
                && !meta.file_type().is_symlink()
                && meta.len() <= INSTALLED_LIMIT =>
        {
            let file =
                fs::File::open(path).map_err(|_| "Cannot read installed application files.")?;
            let bytes = bounded_read(
                file,
                INSTALLED_LIMIT,
                Instant::now() + Duration::from_secs(15),
            )?;
            Ok(Some(sha256(&bytes)))
        }
        _ => Err("Update destination is not a regular application file.".into()),
    }
}

fn safe_destination(root: &Path, relative: &str) -> Result<PathBuf> {
    if fs::symlink_metadata(root).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err("Update destination contains a symbolic link.".into());
    }
    let mut path = root.to_owned();
    for part in Path::new(relative).components() {
        let Component::Normal(part) = part else {
            return Err("Unsafe update archive path.".into());
        };
        path.push(part);
        if fs::symlink_metadata(&path).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err("Update destination contains a symbolic link.".into());
        }
    }
    Ok(path)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstallPlan {
    pub root: PathBuf,
    pub target: String,
    pub before: BTreeMap<String, Option<String>>,
    pub after: BTreeMap<String, String>,
    pub parent_pid: u32,
    pub language: Language,
}

pub struct Staged {
    pub directory: TempDir,
    pub plan: InstallPlan,
}

pub fn stage(update: &Update, language: Language) -> Result<Staged> {
    let exe = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|_| "Cannot locate the installed application.")?;
    let root = if cfg!(target_os = "macos") {
        exe.parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .filter(|p| p.extension().is_some_and(|e| e == "app"))
    } else {
        exe.parent()
    }
    .ok_or("Run pp-switch from its installed package to update it.")?
    .to_owned();
    let marker = if cfg!(target_os = "macos") {
        root.join("Contents/Info.plist")
    } else {
        root.join("README.txt")
    };
    let marker =
        storage::read(&marker)?.ok_or("Run pp-switch from its installed package to update it.")?;
    if if cfg!(target_os = "macos") {
        !marker.contains("<string>io.github.phonowell.pp-switch</string>")
    } else {
        !marker.starts_with("pp-switch ")
    } {
        return Err("Run pp-switch from its installed package to update it.".into());
    }
    let _probe = NamedTempFile::new_in(&root).map_err(|_| "Application directory is not writable. Move it to a user-writable location and try again.")?;
    let before = owned_files(target())
        .into_iter()
        .map(|name| Ok((name.to_owned(), file_hash(&safe_destination(&root, name)?)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let deadline = Instant::now() + Duration::from_secs(45);
    let bytes = bounded_read(
        response(&update.url, deadline)?.into_reader(),
        ARCHIVE_LIMIT,
        deadline,
    )?;
    if bytes.len() as u64 != update.size || sha256(&bytes) != update.digest {
        return Err("Update checksum mismatch. Nothing was installed.".into());
    }
    let files = unpack(&bytes, &update.version, target())?;
    let directory = tempfile::Builder::new()
        .prefix("pp-switch-update-")
        .tempdir()
        .map_err(|_| "Cannot create update staging directory.")?;
    // TempDir is already 0700 on Unix. File-only 0600 would make it untraversable.
    #[cfg(windows)]
    storage::private_file(directory.path())?;
    let mut plan = InstallPlan {
        root,
        target: target().into(),
        before,
        after: BTreeMap::new(),
        parent_pid: std::process::id(),
        language,
    };
    for (name, bytes) in files {
        let destination = safe_destination(&plan.root, &name)?;
        if plan.before.get(&name) != Some(&file_hash(&destination)?) {
            return Err(
                "Application files changed during the update. Nothing was installed.".into(),
            );
        }
        plan.after.insert(name.clone(), sha256(&bytes));
        let staged = directory.path().join("payload").join(&name);
        fs::create_dir_all(staged.parent().unwrap()).map_err(|_| "Cannot stage update files.")?;
        fs::write(&staged, bytes).map_err(|_| "Cannot stage update files.")?;
        storage::private_file(&staged)?;
    }
    Ok(Staged { directory, plan })
}

fn rollback(applied: &mut Vec<(PathBuf, Option<NamedTempFile>, String)>) -> bool {
    let mut restored = true;
    for (path, backup, after) in applied.drain(..).rev() {
        if file_hash(&path).ok().flatten().as_ref() != Some(&after) {
            if let Some(backup) = backup {
                let _ = backup.keep();
            }
            restored = false;
            continue;
        }
        if let Some(backup) = backup {
            if let Err(error) = backup.persist(path) {
                let _ = error.file.keep();
                restored = false;
            }
        } else if fs::remove_file(path).is_err() {
            restored = false;
        }
    }
    restored
}

/// Real filesystem transaction, also exercised against isolated installation fixtures.
pub fn install(plan: &InstallPlan, staging: &Path) -> Result<PathBuf> {
    install_with_hook(plan, staging, |_| Ok(()))
}

fn install_with_hook(
    plan: &InstallPlan,
    staging: &Path,
    mut before_write: impl FnMut(usize) -> Result<()>,
) -> Result<PathBuf> {
    let names = owned_files(&plan.target);
    if plan.target != target()
        || plan.before.len() != names.len()
        || plan.after.len() != names.len()
        || !plan.root.is_absolute()
        || fs::canonicalize(&plan.root).ok().as_ref() != Some(&plan.root)
    {
        return Err("Invalid update installation plan.".into());
    }
    for name in &names {
        let path = safe_destination(&plan.root, name)?;
        if plan.before.get(*name) != Some(&file_hash(&path)?) {
            return Err(
                "Application files changed during the update. Nothing was installed.".into(),
            );
        }
        let source = safe_destination(&staging.join("payload"), name)?;
        if file_hash(&source)?.as_ref() != plan.after.get(*name) {
            return Err("Update checksum mismatch. Nothing was installed.".into());
        }
    }
    let mut applied = vec![];
    let operation = (|| -> Result<()> {
        for (index, name) in names.iter().enumerate() {
            before_write(index)?;
            let path = safe_destination(&plan.root, name)?;
            if plan.before.get(*name) != Some(&file_hash(&path)?) {
                return Err(
                    "Application files changed during the update. Nothing was installed.".into(),
                );
            }
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent).map_err(|_| "Cannot install update files.")?;
            let mut replacement =
                NamedTempFile::new_in(parent).map_err(|_| "Cannot install update files.")?;
            let source = safe_destination(&staging.join("payload"), name)?;
            fs::copy(source, replacement.path()).map_err(|_| "Cannot install update files.")?;
            if file_hash(replacement.path())?.as_ref() != plan.after.get(*name) {
                return Err("Update checksum mismatch. Nothing was installed.".into());
            }
            let backup = if path.exists() {
                let copy = NamedTempFile::new_in(parent)
                    .map_err(|_| "Cannot back up application files.")?;
                fs::copy(&path, copy.path()).map_err(|_| "Cannot back up application files.")?;
                copy.as_file()
                    .sync_all()
                    .map_err(|_| "Cannot back up application files.")?;
                if file_hash(copy.path())?.as_ref() != plan.before[*name].as_ref() {
                    return Err(
                        "Application files changed during the update. Nothing was installed."
                            .into(),
                    );
                }
                Some(copy)
            } else {
                None
            };
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = fs::metadata(&path)
                    .map(|m| m.permissions().mode())
                    .unwrap_or(if *name == *names.last().unwrap() {
                        0o755
                    } else {
                        0o644
                    });
                fs::set_permissions(replacement.path(), fs::Permissions::from_mode(mode))
                    .map_err(|_| "Cannot install update files.")?;
            }
            replacement
                .flush()
                .and_then(|_| replacement.as_file().sync_all())
                .map_err(|_| "Cannot install update files.")?;
            if plan.before.get(*name) != Some(&file_hash(&path)?) {
                return Err(
                    "Application files changed during the update. Nothing was installed.".into(),
                );
            }
            replacement
                .persist(&path)
                .map_err(|_| "Cannot install update files.")?;
            applied.push((path, backup, plan.after[*name].clone()));
        }
        Ok(())
    })();
    if operation.is_err() {
        return Err(if rollback(&mut applied) {
            "Update installation failed. Previous version was restored."
        } else {
            "Update rollback conflicted with external changes. Backup files were preserved."
        }
        .into());
    }
    Ok(plan.root.join(names.last().unwrap()))
}

pub fn launch_helper(staged: Staged) -> Result<()> {
    let directory = staged.directory.path();
    let helper = directory.join(if cfg!(windows) {
        "installer.exe"
    } else {
        "installer"
    });
    fs::copy(
        std::env::current_exe().map_err(|_| "Cannot locate the installed application.")?,
        &helper,
    )
    .map_err(|_| "Cannot start update installer.")?;
    storage::private_file(&helper)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Cannot start update installer.")?;
    }
    let manifest = directory.join("install.json");
    fs::write(
        &manifest,
        serde_json::to_vec(&staged.plan).map_err(|_| "Invalid update installation plan.")?,
    )
    .map_err(|_| "Cannot start update installer.")?;
    storage::private_file(&manifest)?;
    Command::new(helper)
        .arg("--pp-install")
        .arg(manifest)
        .spawn()
        .map_err(|_| "Cannot start update installer.")?;
    let _ = staged.directory.keep();
    Ok(())
}

fn wait_for_exit(pid: u32, timeout: Duration) -> Result<()> {
    if pid <= 1 || pid > i32::MAX as u32 || pid == std::process::id() {
        return Err("Invalid update installation plan.".into());
    }
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_TIMEOUT},
            System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
        };
        let process = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if process.is_null() {
            return Ok(());
        }
        let status = WaitForSingleObject(process, timeout.as_millis() as u32);
        CloseHandle(process);
        if status == WAIT_TIMEOUT {
            return Err("Close other pp-switch windows and try updating again.".into());
        }
        Ok(())
    }
    #[cfg(unix)]
    {
        let deadline = Instant::now() + timeout;
        while unsafe { libc::kill(pid as i32, 0) } == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
        {
            if Instant::now() >= deadline {
                return Err("Close other pp-switch windows and try updating again.".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }
}

fn staging_directory(path: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(path).map_err(|_| "Invalid update installation plan.")?;
    let temp =
        fs::canonicalize(std::env::temp_dir()).map_err(|_| "Invalid update installation plan.")?;
    if path.parent() != Some(temp.as_path())
        || !path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("pp-switch-update-"))
    {
        return Err("Invalid update installation plan.".into());
    }
    Ok(path)
}

fn restart(plan: &InstallPlan, stage: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("/usr/bin/open");
        command.arg("-n").arg(&plan.root);
        command
    };
    #[cfg(not(target_os = "macos"))]
    let mut command = Command::new(plan.root.join(owned_files(&plan.target).last().unwrap()));
    #[cfg(windows)]
    command
        .arg("--pp-clean-update")
        .arg(stage)
        .arg(std::process::id().to_string());
    #[cfg(not(windows))]
    let _ = stage;
    command
        .spawn()
        .map_err(|_| "Update installed. Please reopen pp-switch manually.")?;
    Ok(())
}

/// Internal installer entry; not a user-facing CLI.
pub fn helper(manifest: &Path) -> Result<()> {
    let stage = staging_directory(
        manifest
            .parent()
            .ok_or("Invalid update installation plan.")?,
    )?;
    let text = storage::read(manifest)?.ok_or("Invalid update installation plan.")?;
    let plan: InstallPlan =
        serde_json::from_str(&text).map_err(|_| "Invalid update installation plan.")?;
    let result = wait_for_exit(plan.parent_pid, Duration::from_secs(30))
        .and_then(|_| install(&plan, &stage));
    let restart_result = if let Err(error) = &result {
        i18n::alert(plan.language, error);
        let exe = *owned_files(&plan.target).last().unwrap();
        if file_hash(&plan.root.join(exe)).ok().as_ref() == plan.before.get(exe) {
            restart(&plan, &stage)
        } else {
            Ok(())
        }
    } else {
        restart(&plan, &stage)
    };
    if let Err(error) = &restart_result {
        i18n::alert(plan.language, error);
    }
    #[cfg(not(windows))]
    {
        let _ = fs::remove_dir_all(stage);
    }
    result.map(|_| ()).and(restart_result)
}

pub fn cleanup(stage: &Path, helper_pid: u32) {
    if let Ok(stage) = staging_directory(stage) {
        let _ = wait_for_exit(helper_pid, Duration::from_secs(5));
        let _ = fs::remove_dir_all(stage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, target: &str) -> serde_json::Value {
        let name = asset_name(&Version::parse(version).unwrap(), target);
        serde_json::json!({"tag_name":format!("v{version}"),"draft":false,"assets":[{
            "name":name,"browser_download_url":format!("https://github.com/{REPOSITORY}/releases/download/v{version}/{name}"),
            "size":100,"digest":format!("sha256:{}", "a".repeat(64))
        }]})
    }

    #[test]
    fn pp15_selects_newest_platform_release_and_rejects_unverified_metadata() {
        let current = Version::parse("0.0.2").unwrap();
        let t = target();
        let mut draft = release("9.0.0", t);
        draft["draft"] = true.into();
        let json = serde_json::json!([
            release("0.0.1", t),
            release("0.0.4", t),
            draft,
            release("0.0.3", t)
        ])
        .to_string();
        assert_eq!(
            select_release(&json, &current, t).unwrap().unwrap().version,
            Version::parse("0.0.4").unwrap()
        );
        assert!(
            select_release(&json, &Version::parse("10.0.0").unwrap(), t)
                .unwrap()
                .is_none()
        );
        for (field, bad) in [
            ("digest", serde_json::Value::Null),
            ("size", 0.into()),
            (
                "browser_download_url",
                "https://example.com/pp-switch.zip".into(),
            ),
        ] {
            let mut r = release("0.0.3", t);
            r["assets"][0][field] = bad;
            assert!(select_release(&serde_json::json!([r]).to_string(), &current, t).is_err());
        }
        let mut r = release("0.0.3", t);
        let duplicate = r["assets"][0].clone();
        r["assets"].as_array_mut().unwrap().push(duplicate);
        assert!(select_release(&serde_json::json!([r]).to_string(), &current, t).is_err());
    }

    fn binary(t: &str) -> Vec<u8> {
        let mut b = vec![0u8; 128];
        if t.contains("apple") {
            b[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
            b[4..8].copy_from_slice(
                &(if t.starts_with("aarch64") {
                    0x0100_000cu32
                } else {
                    0x0100_0007u32
                })
                .to_le_bytes(),
            );
        } else if t.contains("windows") {
            b[..2].copy_from_slice(b"MZ");
            b[0x3c..0x40].copy_from_slice(&64u32.to_le_bytes());
            b[64..70].copy_from_slice(&[b'P', b'E', 0, 0, 0x64, 0x86]);
        } else {
            b[..6].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1]);
            b[18..20].copy_from_slice(&[62, 0]);
        }
        b
    }

    fn archive(t: &str, extra: Option<(&str, bool)>, wrong_binary: bool) -> Vec<u8> {
        let v = Version::parse("0.0.3").unwrap();
        let prefix = if t.contains("apple") {
            "pp-switch.app/".to_owned()
        } else if t.contains("linux") {
            format!("pp-switch-v{v}-{t}/")
        } else {
            String::new()
        };
        let mut files: Vec<_> = owned_files(t)
            .iter()
            .map(|name| (format!("{prefix}{name}"), b"example".to_vec()))
            .collect();
        files.last_mut().unwrap().1 = if wrong_binary {
            b"wrong-platform".to_vec()
        } else {
            binary(t)
        };
        if t.contains("linux") {
            let mut tar = tar::Builder::new(Vec::new());
            for (name, data) in &files {
                let mut h = tar::Header::new_gnu();
                h.set_size(data.len() as u64);
                h.set_mode(0o644);
                h.set_cksum();
                tar.append_data(&mut h, name, &data[..]).unwrap();
            }
            if let Some((name, dir)) = extra {
                let mut h = tar::Header::new_gnu();
                h.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
                h.set_entry_type(if dir {
                    tar::EntryType::Directory
                } else {
                    tar::EntryType::Symlink
                });
                h.set_size(0);
                h.set_mode(0o755);
                h.set_cksum();
                tar.append(&h, std::io::empty()).unwrap();
            }
            let tar = tar.into_inner().unwrap();
            let mut gzip =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            gzip.write_all(&tar).unwrap();
            gzip.finish().unwrap()
        } else {
            let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
            let options = zip::write::SimpleFileOptions::default();
            for (name, data) in &files {
                zip.start_file(name, options).unwrap();
                zip.write_all(data).unwrap();
            }
            if let Some((name, dir)) = extra {
                if dir {
                    zip.add_directory(name, options).unwrap();
                } else {
                    zip.add_symlink(name, "outside", options).unwrap();
                }
            }
            zip.finish().unwrap().into_inner()
        }
    }

    #[test]
    fn pp15_archive_whitelist_paths_links_and_architecture() {
        let version = Version::parse("0.0.3").unwrap();
        for t in [
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "x86_64-pc-windows-msvc",
            "x86_64-unknown-linux-gnu",
        ] {
            assert_eq!(
                unpack(&archive(t, None, false), &version, t).unwrap().len(),
                owned_files(t).len()
            );
            assert!(unpack(&archive(t, None, true), &version, t).is_err());
            for extra in [("../escape/", true), ("unrelated/", true), ("link", false)] {
                assert!(
                    unpack(&archive(t, Some(extra), false), &version, t).is_err(),
                    "{t} {extra:?}"
                );
            }
        }
        assert!(unpack(b"not an archive", &version, target()).is_err());
    }

    fn installation() -> (TempDir, TempDir, InstallPlan) {
        let root = tempfile::tempdir().unwrap();
        let stage = tempfile::tempdir().unwrap();
        let mut plan = InstallPlan {
            root: fs::canonicalize(root.path()).unwrap(),
            target: target().into(),
            before: BTreeMap::new(),
            after: BTreeMap::new(),
            parent_pid: 2,
            language: Language::En,
        };
        for name in owned_files(target()) {
            let path = plan.root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"old").unwrap();
            let staged = stage.path().join("payload").join(name);
            fs::create_dir_all(staged.parent().unwrap()).unwrap();
            fs::write(staged, b"new").unwrap();
            plan.before.insert(name.into(), Some(sha256(b"old")));
            plan.after.insert(name.into(), sha256(b"new"));
        }
        fs::write(plan.root.join("user-file"), b"unrelated").unwrap();
        (root, stage, plan)
    }

    #[test]
    fn pp15_installs_owned_files_and_preserves_unrelated_files() {
        let (_root, stage, plan) = installation();
        install(&plan, stage.path()).unwrap();
        for name in owned_files(target()) {
            assert_eq!(fs::read(plan.root.join(name)).unwrap(), b"new");
        }
        assert_eq!(fs::read(plan.root.join("user-file")).unwrap(), b"unrelated");
    }

    #[test]
    fn pp15_fault_after_first_replace_restores_previous_version() {
        let (_root, stage, plan) = installation();
        assert!(
            install_with_hook(&plan, stage.path(), |index| if index == 1 {
                Err("Injected failure".into())
            } else {
                Ok(())
            })
            .unwrap_err()
            .contains("Previous version was restored")
        );
        for name in owned_files(target()) {
            assert_eq!(fs::read(plan.root.join(name)).unwrap(), b"old");
        }
    }

    #[test]
    fn pp15_rollback_preserves_external_changes_and_backups() {
        let (_root, stage, plan) = installation();
        let first = plan.root.join(owned_files(target())[0]);
        let error = install_with_hook(&plan, stage.path(), |index| {
            if index == 1 {
                fs::write(&first, b"external").unwrap();
                Err("Injected failure".into())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert!(error.contains("Backup files were preserved"));
        assert_eq!(fs::read(first).unwrap(), b"external");
        assert!(
            fs::read_dir(plan.root.join(owned_files(target())[0]).parent().unwrap())
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().starts_with(".tmp")
                    && fs::read(e.path()).ok().as_deref() == Some(b"old"))
        );
    }

    #[test]
    fn pp15_changed_destination_or_tampered_payload_never_installs() {
        for payload in [false, true] {
            let (_root, stage, plan) = installation();
            let name = owned_files(target())[0];
            let changed = if payload {
                stage.path().join("payload").join(name)
            } else {
                plan.root.join(name)
            };
            fs::write(changed, b"tampered").unwrap();
            assert!(install(&plan, stage.path()).is_err());
            for name in owned_files(target()).into_iter().skip(1) {
                assert_eq!(fs::read(plan.root.join(name)).unwrap(), b"old");
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn pp15_symlink_destination_and_staging_are_refused() {
        use std::os::unix::fs::symlink;
        for payload in [false, true] {
            let (_root, stage, plan) = installation();
            let outside = tempfile::NamedTempFile::new().unwrap();
            fs::write(outside.path(), b"outside").unwrap();
            let name = owned_files(target())[0];
            let path = if payload {
                stage.path().join("payload").join(name)
            } else {
                plan.root.join(name)
            };
            fs::remove_file(&path).unwrap();
            symlink(outside.path(), &path).unwrap();
            assert!(install(&plan, stage.path()).is_err());
            assert_eq!(fs::read(outside.path()).unwrap(), b"outside");
            assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
        }
    }
}
