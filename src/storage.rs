//! Bounded local storage and a recoverable, cooperative file transaction.
use crate::{Result, i18n::Language, provider::Provider};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAX_FILE: u64 = 1024 * 1024;

#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Change {
    pub path: PathBuf,
    pub before: Option<String>,
    pub after: String,
}

#[derive(Deserialize, Serialize)]
struct Journal {
    changes: Vec<Change>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Preferences {
    language: Language,
}

pub fn read(path: &Path) -> Result<Option<String>> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(format!("Cannot read {}.", path.display())),
    };
    if file
        .metadata()
        .map_err(|_| "Cannot inspect configuration file.")?
        .len()
        > MAX_FILE
    {
        return Err("Configuration file exceeds the 1 MiB limit.".into());
    }
    let mut text = String::new();
    Read::by_ref(&mut file)
        .take(MAX_FILE + 1)
        .read_to_string(&mut text)
        .map_err(|_| "Cannot read UTF-8 configuration.")?;
    if text.len() as u64 > MAX_FILE {
        return Err("Configuration file exceeds the 1 MiB limit.".into());
    }
    Ok(Some(text))
}

pub fn private_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|_| "Cannot restrict credential-file permissions.")?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SE_FILE_OBJECT,
            SetNamedSecurityInfoW,
        };
        use windows_sys::Win32::Security::{
            DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl,
            PROTECTED_DACL_SECURITY_INFORMATION,
        };
        let sid = current_user_sid()?;
        let sddl: Vec<u16> = format!("D:P(A;;FA;;;{sid})\0").encode_utf16().collect();
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut descriptor = std::ptr::null_mut();
        // Protected DACL: only the current user SID has access. No shell or subprocess.
        unsafe {
            if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err("Cannot create a current-user credential ACL.".into());
            }
            let mut present = 0;
            let mut defaulted = 0;
            let mut acl = std::ptr::null_mut();
            let got = GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted);
            let result = if got != 0 && present != 0 {
                SetNamedSecurityInfoW(
                    path.as_ptr(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    acl,
                    std::ptr::null_mut(),
                )
            } else {
                1
            };
            LocalFree(descriptor);
            if result != 0 {
                return Err("Cannot restrict credential-file ACL to the current user.".into());
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn current_user_sid() -> Result<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree};
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err("Cannot identify the current user for credential permissions.".into());
        }
        let mut length = 0;
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut length);
        if length == 0 || length > 65536 {
            CloseHandle(token);
            return Err("Cannot read current-user security information.".into());
        }
        let mut buffer = vec![0usize; (length as usize).div_ceil(std::mem::size_of::<usize>())];
        let result = GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            length,
            &mut length,
        );
        CloseHandle(token);
        if result == 0 {
            return Err("Cannot read current-user security information.".into());
        }
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        let mut text = std::ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut text) == 0 {
            return Err("Cannot encode current-user security identifier.".into());
        }
        let mut length = 0;
        while *text.add(length) != 0 {
            length += 1;
        }
        let sid = String::from_utf16(std::slice::from_raw_parts(text, length));
        LocalFree(text.cast());
        sid.map_err(|_| "Invalid current-user security identifier.".into())
    }
}

fn write_atomic(path: &Path, text: &str) -> Result<()> {
    writable_destination(path)?;
    let parent = path
        .parent()
        .ok_or("Configuration path has no parent directory.")?;
    fs::create_dir_all(parent).map_err(|_| "Cannot create configuration directory.")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|_| "Cannot create temporary configuration file.")?;
    private_file(temp.path())?;
    temp.write_all(text.as_bytes())
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|_| "Cannot flush configuration file.")?;
    temp.persist(path)
        .map_err(|_| "Cannot replace configuration file.")?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(|_| "Cannot flush configuration directory.")?;
    Ok(())
}

fn writable_destination(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err("Configuration file is a symbolic link. pp-switch will not replace the link; point the client config directory at real files.".into()),
        Ok(meta) if !meta.is_file() => Err("Configuration destination is not a regular file.".into()),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Cannot inspect configuration destination.".into()),
    }
}

impl Store {
    pub fn language(&self) -> Result<Language> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        match read(&self.root.join("ui.json"))? {
            None => Ok(Language::system()),
            Some(text) => serde_json::from_str::<Preferences>(&text)
                .map(|prefs| prefs.language)
                .map_err(|_| "Language preferences are invalid; they were not overwritten.".into()),
        }
    }

    pub fn save_language(&self, language: Language) -> Result<()> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        let path = self.root.join("ui.json");
        let before = read(&path)?;
        if let Some(text) = &before {
            serde_json::from_str::<Preferences>(text)
                .map_err(|_| "Language preferences are invalid; they were not overwritten.")?;
        }
        let after = serde_json::to_string(&Preferences { language })
            .map_err(|_| "Cannot encode language preferences.")?;
        self.commit_locked(vec![Change {
            path,
            before,
            after,
        }])
    }

    pub fn system() -> Result<Self> {
        Ok(Self {
            root: dirs::data_local_dir()
                .ok_or("Cannot locate user application-data directory.")?
                .join("pp-switch"),
        })
    }

    fn lock(&self) -> Result<File> {
        fs::create_dir_all(&self.root).map_err(|_| "Cannot create pp-switch data directory.")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.root, fs::Permissions::from_mode(0o700))
                .map_err(|_| "Cannot restrict data-directory permissions.")?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join("lock"))
            .map_err(|_| "Cannot open pp-switch write lock.")?;
        lock.try_lock()
            .map_err(|_| "Another pp-switch operation is writing. Try again.")?;
        Ok(lock)
    }

    fn recover_locked(&self) -> Result<()> {
        let path = self.root.join("pending.json");
        let Some(text) = read(&path)? else {
            return Ok(());
        };
        let journal: Journal = serde_json::from_str(&text).map_err(
            |_| "Recovery journal is invalid. Preserve it and inspect the data directory.",
        )?;
        if journal.changes.len() > 4 {
            return Err("Recovery journal contains too many files.".into());
        }
        for change in &journal.changes {
            writable_destination(&change.path)?;
            let current = read(&change.path)?;
            if current != change.before && current.as_deref() != Some(&change.after) {
                return Err("A file changed outside pp-switch during recovery. No files were overwritten; preserve pending.json and resolve the conflict.".into());
            }
        }
        for change in &journal.changes {
            if read(&change.path)? == change.before {
                continue;
            }
            if let Some(before) = &change.before {
                write_atomic(&change.path, before)?;
            } else {
                fs::remove_file(&change.path)
                    .map_err(|_| "Cannot restore a previously absent file.")?;
            }
        }
        fs::remove_file(path).map_err(|_| "Cannot clear recovery journal.")?;
        Ok(())
    }

    pub fn recover(&self) -> Result<()> {
        let _lock = self.lock()?;
        self.recover_locked()
    }

    fn commit_locked(&self, changes: Vec<Change>) -> Result<()> {
        let changes: Vec<_> = changes
            .into_iter()
            .filter(|c| c.before.as_deref() != Some(c.after.as_str()))
            .collect();
        if changes.is_empty() {
            return Ok(());
        }
        if changes.len() > 4 {
            return Err("Too many files in one operation.".into());
        }
        for change in &changes {
            writable_destination(&change.path)?;
            if read(&change.path)? != change.before {
                return Err(
                    "Configuration changed while preparing this operation. Reload and try again."
                        .into(),
                );
            }
        }
        let journal = Journal { changes };
        let text =
            serde_json::to_string(&journal).map_err(|_| "Cannot encode recovery journal.")?;
        if text.len() as u64 > MAX_FILE {
            return Err("Operation exceeds the recovery journal size limit.".into());
        }
        let path = self.root.join("pending.json");
        write_atomic(&path, &text)?;
        let operation = (|| {
            for change in &journal.changes {
                if read(&change.path)? != change.before {
                    return Err("Configuration changed during this operation.".into());
                }
                write_atomic(&change.path, &change.after)?;
            }
            for change in &journal.changes {
                if read(&change.path)?.as_deref() != Some(&change.after) {
                    return Err("Configuration changed before verification finished.".into());
                }
            }
            fs::remove_file(path).map_err(|_| "Cannot finish the recovery journal.".to_string())
        })();
        if operation.is_err() {
            self.recover_locked()?;
            return Err("Write failed. Previous configuration was restored.".into());
        }
        Ok(())
    }

    pub fn commit(&self, changes: Vec<Change>) -> Result<()> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        self.commit_locked(changes)
    }

    /// Validate read-only configuration inputs under the same write lock.
    pub(crate) fn commit_guarded(
        &self,
        changes: Vec<Change>,
        guards: &[(PathBuf, Option<String>)],
    ) -> Result<()> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        for (path, expected) in guards {
            if read(path)? != *expected {
                return Err(
                    "Configuration changed while preparing this operation. Reload and try again."
                        .into(),
                );
            }
        }
        self.commit_locked(changes)
    }

    fn decode_providers(text: Option<&str>) -> Result<Vec<Provider>> {
        let providers: Vec<Provider> = serde_json::from_str(text.unwrap_or("[]"))
            .map_err(|_| "Provider store is invalid; it was not overwritten.")?;
        if providers.len() > 128 {
            return Err("Provider store exceeds the 128-provider limit.".into());
        }
        let mut ids = std::collections::HashSet::new();
        for p in &providers {
            p.validate()?;
            if p.id.is_empty() || !ids.insert(&p.id) {
                return Err("Provider store contains invalid or duplicate IDs.".into());
            }
        }
        Ok(providers)
    }

    pub fn load(&self) -> Result<Vec<Provider>> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        Self::decode_providers(read(&self.root.join("providers.json"))?.as_deref())
    }

    fn update(&self, edit: impl FnOnce(&mut Vec<Provider>) -> Result<()>) -> Result<()> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        let path = self.root.join("providers.json");
        let before = read(&path)?;
        let mut providers = Self::decode_providers(before.as_deref())?;
        edit(&mut providers)?;
        if providers.len() > 128 {
            return Err("Maximum 128 saved providers.".into());
        }
        let after = serde_json::to_string_pretty(&providers)
            .map_err(|_| "Cannot encode provider store.")?;
        self.commit_locked(vec![Change {
            path,
            before,
            after,
        }])
    }

    pub fn save(&self, provider: Provider) -> Result<()> {
        provider.validate()?;
        self.update(|providers| {
            if providers.iter().any(|p| p.id != provider.id && p.same_name(provider.client, &provider.name)) {
                return Err("A provider with this name already exists for this client. Choose a different name.".into());
            }
            if let Some(existing) = providers.iter_mut().find(|p| p.id == provider.id) {
                *existing = provider;
            } else {
                providers.push(provider);
            }
            Ok(())
        })
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.update(|providers| {
            providers.retain(|p| p.id != id);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pp09_changed_read_only_guard_prevents_every_destination_write() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            root: dir.path().join("store"),
        };
        let input = dir.path().join("profile.yml");
        let destination = dir.path().join("home.yml");
        fs::write(&input, "external change").unwrap();
        fs::write(&destination, "old").unwrap();
        assert!(
            store
                .commit_guarded(
                    vec![Change {
                        path: destination.clone(),
                        before: Some("old".into()),
                        after: "new".into()
                    }],
                    &[(input.clone(), Some("snapshot".into()))]
                )
                .is_err()
        );
        assert_eq!(fs::read_to_string(&input).unwrap(), "external change");
        assert_eq!(fs::read_to_string(&destination).unwrap(), "old");
        assert!(!store.root.join("pending.json").exists());
    }
}
