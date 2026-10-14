//! Worker lifecycle shared by the native UI and background behavior tests.
use crate::{Result, i18n::Language, update};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Instant,
};

enum Event {
    Checked(Result<Option<update::Update>>),
    Staged(Result<update::Staged>),
}

pub(super) enum Progress {
    Waiting,
    Confirm(update::Update),
    Finished(Result<&'static str>),
    Installed,
}

pub(super) struct Job {
    receiver: Receiver<Event>,
    deadline: Instant,
    stage: fn(&update::Update, Language) -> Result<update::Staged>,
    launch: fn(update::Staged) -> Result<()>,
}

impl Job {
    pub(super) fn start(
        check: fn() -> Result<Option<update::Update>>,
        stage: fn(&update::Update, Language) -> Result<update::Staged>,
        launch: fn(update::Staged) -> Result<()>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(Event::Checked(check()));
        });
        Self {
            receiver,
            deadline: Instant::now() + update::UI_TIMEOUT,
            stage,
            launch,
        }
    }

    pub(super) fn confirm(
        &mut self,
        release: update::Update,
        accepted: bool,
        language: Language,
        now: Instant,
    ) -> Progress {
        if !accepted {
            return Progress::Finished(Ok("Update cancelled. Nothing was installed."));
        }
        let (sender, receiver) = mpsc::channel();
        self.receiver = receiver;
        self.deadline = now + update::UI_TIMEOUT;
        let stage = self.stage;
        std::thread::spawn(move || {
            let _ = sender.send(Event::Staged(stage(&release, language)));
        });
        Progress::Waiting
    }

    pub(super) fn poll(&mut self, now: Instant, language: Language) -> Progress {
        // Consume an already completed worker before considering its watchdog.
        match self.receiver.try_recv() {
            Ok(Event::Checked(Ok(Some(release)))) => Progress::Confirm(release),
            Ok(Event::Checked(Ok(None))) => {
                Progress::Finished(Ok("You are using the latest available version."))
            }
            Ok(Event::Checked(Err(error)) | Event::Staged(Err(error))) => {
                Progress::Finished(Err(error))
            }
            Ok(Event::Staged(Ok(mut staged))) => {
                staged.plan.language = language;
                match (self.launch)(staged) {
                    Ok(()) => Progress::Installed,
                    Err(error) => Progress::Finished(Err(error)),
                }
            }
            Err(TryRecvError::Disconnected) => {
                Progress::Finished(Err("Update worker stopped. Try again.".into()))
            }
            Err(TryRecvError::Empty) if now >= self.deadline => {
                Progress::Finished(Err("Update request timed out. Try again.".into()))
            }
            Err(TryRecvError::Empty) => Progress::Waiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, time::Duration};

    fn offer() -> Result<Option<update::Update>> {
        Ok(Some(update::Update {
            version: semver::Version::new(1, 0, 0),
            url: "https://github.com/phonowell/pp-switch/fixture".into(),
            digest: "a".repeat(64),
            size: 100,
        }))
    }

    fn stage(release: &update::Update, language: Language) -> Result<update::Staged> {
        assert_eq!(release.version, semver::Version::new(1, 0, 0));
        assert_eq!(language, Language::Zh);
        let directory = tempfile::tempdir().unwrap();
        let plan = update::InstallPlan {
            root: directory.path().to_owned(),
            target: update::target().into(),
            before: BTreeMap::new(),
            after: BTreeMap::new(),
            parent_pid: std::process::id(),
            language,
        };
        Ok(update::Staged { directory, plan })
    }

    fn launch(staged: update::Staged) -> Result<()> {
        assert!(staged.directory.path().exists());
        // Language can change while a worker is running.
        assert_eq!(staged.plan.language, Language::Ja);
        Ok(())
    }

    fn await_progress(job: &mut Job) -> Progress {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let progress = job.poll(Instant::now(), Language::Ja);
            if !matches!(progress, Progress::Waiting) {
                return progress;
            }
            assert!(Instant::now() < deadline, "fixture worker did not finish");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn accept(job: &mut Job, accepted: bool) -> Progress {
        let Progress::Confirm(release) = await_progress(job) else {
            panic!("expected a confirmation before staging");
        };
        job.confirm(release, accepted, Language::Zh, Instant::now())
    }

    #[test]
    fn pp15_check_confirm_stage_launch_permits_quit_only_after_launch() {
        let mut job = Job::start(offer, stage, launch);
        assert!(matches!(accept(&mut job, true), Progress::Waiting));
        assert!(matches!(await_progress(&mut job), Progress::Installed));
    }

    #[test]
    fn pp15_cancel_never_stages_or_launches() {
        let mut job = Job::start(
            offer,
            |_, _| panic!("cancelled stage"),
            |_| panic!("cancelled launch"),
        );
        assert!(matches!(
            accept(&mut job, false),
            Progress::Finished(Ok("Update cancelled. Nothing was installed."))
        ));
    }

    #[test]
    fn pp15_errors_finish_without_quit() {
        let mut job = Job::start(|| Ok(None), stage, launch);
        assert!(matches!(
            await_progress(&mut job),
            Progress::Finished(Ok("You are using the latest available version."))
        ));
        let mut job = Job::start(|| Err("connection failure".into()), stage, launch);
        assert!(
            matches!(await_progress(&mut job), Progress::Finished(Err(error)) if error == "connection failure")
        );
        let mut job = Job::start(
            offer,
            |_, _| Err("disk full".into()),
            |_| panic!("failed stage launched"),
        );
        accept(&mut job, true);
        assert!(
            matches!(await_progress(&mut job), Progress::Finished(Err(error)) if error == "disk full")
        );
        let mut job = Job::start(offer, stage, |_| Err("locked installer".into()));
        accept(&mut job, true);
        assert!(
            matches!(await_progress(&mut job), Progress::Finished(Err(error)) if error == "locked installer")
        );
    }

    #[test]
    fn pp15_watchdog_and_disconnected_worker_finish_and_staging_resets_deadline() {
        let (sender, receiver) = mpsc::channel();
        let now = Instant::now();
        let mut job = Job {
            receiver,
            deadline: now,
            stage,
            launch,
        };
        assert!(
            matches!(job.poll(now, Language::Zh), Progress::Finished(Err(error)) if error.contains("timed out"))
        );
        // The slow confirmation is outside the staging watchdog.
        let later = now + Duration::from_secs(120);
        job.confirm(offer().unwrap().unwrap(), true, Language::Zh, later);
        assert_eq!(job.deadline, later + update::UI_TIMEOUT);
        assert!(matches!(await_progress(&mut job), Progress::Installed));
        drop(sender);
        let (sender, receiver) = mpsc::channel();
        drop(sender);
        job.receiver = receiver;
        assert!(
            matches!(job.poll(now, Language::Zh), Progress::Finished(Err(error)) if error.contains("worker stopped"))
        );
    }
}
