use std::path::PathBuf;

use git2::Repository;

struct GitWorkflow {
  raw: Repository,
  pub root: PathBuf,
}
impl GitWorkflow {
  pub fn prepare(&self) {

  }

  pub fn staged_files(&self) -> Vec<PathBuf> {
      let repo = &self.raw;
      let head_tree = repo.head().unwrap().peel_to_tree().unwrap();
      let diff = repo
          .diff_tree_to_index(Some(&head_tree), None, None)
          .unwrap();
      let mut staged_files = diff
          .deltas()
          .flat_map(|delta| vec![delta.old_file().path(), delta.new_file().path()])
          .filter_map(std::convert::identity)
          .map(|path| self.root.join(path))
          .collect::<Vec<_>>();
      staged_files.dedup();
      staged_files
  }
}

impl std::fmt::Debug for GitWorkflow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("Repo").finish()
  }
}
