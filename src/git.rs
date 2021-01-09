use anyhow::{bail, Context, Result};
use git2::{Index, IndexAddOption, Oid, Repository, Signature};

pub struct Git {
    repo: Repository,
    index: Index,
}

impl Git {
    pub fn init(repo_path: &str) -> Result<Self> {
        let repo = match Repository::open(&repo_path) {
            Ok(repo) => repo,
            Err(_) => match Repository::init(&repo_path) {
                Ok(repo) => repo,
                Err(e) => bail!("failed to open: {}", e),
            },
        };
        Ok(Self {
            index: repo.index()?,
            repo,
        })
    }

    pub fn add_all(&mut self) -> Result<()> {
        self.index
            .add_all(["*"].iter(), IndexAddOption::CHECK_PATHSPEC, None)?;
        self.index.write()?;
        Ok(())
    }

    pub fn has_diff(&mut self) -> Result<bool> {
        if let Ok(head) = self.repo.head() {
            let diff = self
                .repo
                .diff_tree_to_index(Some(&head.peel_to_tree()?), Some(&self.index), None)?
                .stats()?;
            Ok(diff.files_changed() + diff.insertions() + diff.deletions() > 0)
        } else {
            bail!("no HEAD")
        }
    }

    pub fn commit(&mut self, sig: &Signature, msg: &str) -> Result<Oid> {
        let tree_id = self.index.write_tree()?;
        let mut parents = Vec::new();
        if let Some(parent) = self
            .repo
            .head()
            .ok()
            .map(|h| h.target().expect("failed to get target"))
        {
            parents.push(self.repo.find_commit(parent)?)
        }
        self.repo
            .commit(
                Some("HEAD"),
                sig,
                sig,
                msg,
                &self.repo.find_tree(tree_id)?,
                &parents.iter().collect::<Vec<_>>(),
            )
            .context("failed to commit")
    }
}
