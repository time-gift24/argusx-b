use crate::traits::vcs::VersionControl;
use crate::types::{Branch, Commit, CommitId, Tag};
use anyhow::{anyhow, Context};
use git2::{Repository, Signature, Sort};
use std::path::Path;
use std::sync::Mutex;

pub struct Git2VersionControl {
    repo: Mutex<Option<Repository>>,
    user_name: String,
    user_email: String,
}

impl Default for Git2VersionControl {
    fn default() -> Self {
        Self::new()
    }
}

impl Git2VersionControl {
    pub fn new() -> Self {
        Self {
            repo: Mutex::new(None),
            user_name: "Git VCS User".to_string(),
            user_email: "git-vcs@localhost".to_string(),
        }
    }

    pub fn with_config(user_name: &str, user_email: &str) -> Self {
        Self {
            repo: Mutex::new(None),
            user_name: user_name.to_string(),
            user_email: user_email.to_string(),
        }
    }

    fn signature(&self) -> Signature<'_> {
        Signature::now(&self.user_name, &self.user_email).unwrap()
    }

    fn open_repo(&self, path: &Path) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        *guard = Some(Repository::open(path).context("Failed to open repository")?);
        Ok(())
    }
}

impl VersionControl for Git2VersionControl {
    fn create_repo(&self, path: &Path) -> anyhow::Result<()> {
        Repository::init(path)
            .map(|_| ())
            .context("Failed to create repository")?;
        self.open_repo(path)
    }

    fn clone_repo(&self, url: &str, path: &Path) -> anyhow::Result<()> {
        let repo = Repository::clone(url, path).context("Failed to clone repository")?;
        let mut guard = self.repo.lock().unwrap();
        *guard = Some(repo);
        Ok(())
    }

    fn add(&self, files: &[&Path]) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let workdir = repo.workdir().ok_or_else(|| anyhow!("No workdir"))?;
        let workdir = workdir
            .canonicalize()
            .unwrap_or_else(|_| workdir.to_path_buf());

        let mut index = repo.index()?;
        for file in files {
            // Canonicalize the file path to handle symlinks (e.g., /var -> /private/var on macOS)
            let file_canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
            // Try to get relative path from workdir
            let relative_path = file_canonical.strip_prefix(&workdir).map_err(|_| {
                anyhow!(
                    "Failed to strip prefix: file={}, workdir={}",
                    file_canonical.display(),
                    workdir.display()
                )
            })?;
            // Ensure it's treated as a relative path
            if relative_path.is_absolute() {
                return Err(anyhow!(
                    "Cannot add absolute path to index: {}",
                    relative_path.display()
                ));
            }
            index.add_path(relative_path)?;
        }
        index.write()?;
        Ok(())
    }

    fn commit(&self, message: &str) -> anyhow::Result<CommitId> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut index = repo.index()?;
        let oid = index.write_tree()?;
        let tree = repo.find_tree(oid)?;

        let sig = self.signature();
        let parent = repo.head().ok().map(|h| h.peel_to_commit()).transpose()?;

        let parents: Vec<&git2::Commit> = parent.iter().collect();
        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;

        Ok(CommitId::new(commit_oid.to_string()))
    }

    fn push(&self, remote: &str, branch: &str) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut remote = repo.find_remote(remote)?;
        remote.push(&[&format!("refs/heads/{}", branch)], None)?;
        Ok(())
    }

    fn pull(&self, remote: &str, branch: &str) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut remote = repo.find_remote(remote)?;
        remote.fetch(&[branch], None, None)?;

        let refname = format!("refs/heads/{}", branch);
        let oid = repo.refname_to_id(&refname)?;
        let commit = repo.find_commit(oid)?;

        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        repo.checkout_tree(commit.tree().unwrap().as_object(), Some(&mut checkout_opts))?;

        Ok(())
    }

    fn create_branch(&self, name: &str) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let head = repo.head()?;
        let target = head.peel_to_commit()?;
        repo.branch(name, &target, false)?;
        Ok(())
    }

    fn switch_branch(&self, name: &str) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let reference = repo.find_branch(name, git2::BranchType::Local)?;
        let commit = reference.get().peel_to_commit()?;

        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        repo.checkout_tree(commit.tree().unwrap().as_object(), Some(&mut checkout_opts))?;
        repo.set_head(&format!("refs/heads/{}", name))?;
        Ok(())
    }

    fn delete_branch(&self, name: &str) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut branch = repo.find_branch(name, git2::BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    fn list_branches(&self) -> anyhow::Result<Vec<Branch>> {
        let guard = self.repo.lock().unwrap();
        let repo = guard
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut branches = Vec::new();
        for branch in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch?;
            let name = branch.name()?.unwrap().to_string();
            let is_head = branch.is_head();
            let target = branch.get().target().unwrap();
            branches.push(Branch {
                name,
                is_head,
                target: CommitId::new(target.to_string()),
            });
        }
        Ok(branches)
    }

    fn log(&self, _path: Option<&Path>, _limit: usize) -> anyhow::Result<Vec<Commit>> {
        let guard = self.repo.lock().unwrap();
        let repo = guard
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(Sort::TIME)?;

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            commits.push(Commit {
                id: CommitId::new(oid.to_string()),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                email: commit.author().email().unwrap_or("").to_string(),
                timestamp: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_default(),
            });
        }
        Ok(commits)
    }

    fn diff(&self, from: &CommitId, to: &CommitId, _path: Option<&Path>) -> anyhow::Result<String> {
        let guard = self.repo.lock().unwrap();
        let repo = guard
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let from_commit = repo.find_commit(from.as_str().parse().unwrap())?;
        let to_commit = repo.find_commit(to.as_str().parse().unwrap())?;

        let from_tree = from_commit.tree()?;
        let to_tree = to_commit.tree()?;

        let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)?;
        let mut diff_output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = line.content();
            diff_output.push_str(std::str::from_utf8(content).unwrap_or(""));
            true
        })?;
        Ok(diff_output)
    }

    fn show(&self, commit: &CommitId) -> anyhow::Result<String> {
        let guard = self.repo.lock().unwrap();
        let repo = guard
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let obj = repo.revparse_single(commit.as_str())?;
        let blob = obj.peel_to_blob()?;
        let content = blob.content();
        Ok(String::from_utf8_lossy(content).to_string())
    }

    fn blame(&self, path: &Path) -> anyhow::Result<String> {
        let guard = self.repo.lock().unwrap();
        let repo = guard
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let workdir = repo.workdir().ok_or_else(|| anyhow!("No workdir"))?;
        let relative_path = path.strip_prefix(workdir).unwrap_or(path);

        // Get file content to count lines
        let blob = repo
            .head()?
            .peel_to_commit()?
            .tree()?
            .get_path(relative_path)?
            .to_object(repo)?
            .peel_to_blob()?;
        let content = blob.content();
        let line_count = std::str::from_utf8(content)
            .map(|s| s.lines().count())
            .unwrap_or(0);
        Ok(format!("{} lines blame", line_count))
    }

    fn rollback(&self, commit_id: &CommitId) -> anyhow::Result<()> {
        self.commit(&format!("Revert to {}", commit_id.as_str()))
            .map(|_| ())
    }

    fn create_tag(&self, name: &str, message: Option<&str>) -> anyhow::Result<()> {
        let mut guard = self.repo.lock().unwrap();
        let repo = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let head = repo.head()?;
        let target = head.peel_to_commit()?;

        if let Some(msg) = message {
            repo.tag(name, target.as_object(), &self.signature(), msg, false)?;
        } else {
            repo.tag_lightweight(name, target.as_object(), false)?;
        }
        Ok(())
    }

    fn list_tags(&self) -> anyhow::Result<Vec<Tag>> {
        let guard = self.repo.lock().unwrap();
        let repo = guard
            .as_ref()
            .ok_or_else(|| anyhow!("No repository opened"))?;

        let mut tags = Vec::new();
        for tag in repo.tag_names(None)?.iter() {
            let name = tag.unwrap().to_string();
            // Use tag_names to get Oid directly
            let tag_oid = repo.refname_to_id(&format!("refs/tags/{}", name))?;
            let reference = repo.find_tag(tag_oid)?;
            let target = reference.target().unwrap().id();
            tags.push(Tag {
                name,
                target: CommitId::new(target.to_string()),
                message: reference.message().map(|s| s.to_string()),
            });
        }
        Ok(tags)
    }
}
