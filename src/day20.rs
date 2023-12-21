use std::str::from_utf8;

use axum::body::Bytes;
use color_eyre::eyre::OptionExt;
use git2::{BranchType, Repository, TreeWalkResult};
use tar::{Archive, EntryType};
use tempfile::TempDir;
use tracing::info;

use crate::cch_error::ReportError;

pub async fn num_files(body: Bytes) -> Result<String, ReportError> {
    let mut archive: Archive<&[u8]> = tar::Archive::new(body.as_ref());
    let total_files = archive
        .entries()?
        .filter_map(|e| {
            let entry = e.ok()?;
            let kind = entry.header().entry_type();
            if let EntryType::Regular = kind {
                Some(())
            } else {
                None
            }
        })
        .count()
        .to_string();
    Ok(total_files)
}

pub async fn size_files(body: Bytes) -> Result<String, ReportError> {
    let mut archive: Archive<&[u8]> = tar::Archive::new(body.as_ref());
    let size_files: u64 = archive
        .entries()?
        .filter_map(|e| {
            let entry = e.ok()?;
            let kind = entry.header().entry_type();
            if let EntryType::Regular = kind {
                Some(entry.size())
            } else {
                None
            }
        })
        .sum();
    Ok(size_files.to_string())
}

#[tracing::instrument(skip(body))]
pub async fn find_cookie(body: Bytes) -> Result<String, ReportError> {
    let mut archive: Archive<&[u8]> = tar::Archive::new(body.as_ref());
    // create the temp directory
    let tmp_dir = TempDir::new()?;
    // assume that it is a .git file
    /*
    let _ = archive
        .entries()?
        .filter_map(|e| {
            let entry = e.ok()?;
            Some(entry)
        })
        .find(|e| {
            let path_name = e.path().unwrap();
            path_name == Path::new(".git")
        })
        .ok_or_eyre("No .git dir found")?;
    */
    // just unpack the whole archive
    archive.unpack(tmp_dir.path())?;
    let dir_entries: Vec<_> = tmp_dir
        .path()
        .read_dir()?
        .filter(|d| d.is_ok())
        .map(|d| d.unwrap().path())
        .collect();
    info!(?dir_entries);
    // get branch christmas
    // get the commit pointed by branch
    // iterate over the parents of the commits
    // each commit we get the tree and
    // walk on the tree
    // every tree entry
    //  check if name is santa.txt
    //  try and get the blob
    //  get the content
    //  if the content contains COOKIE
    //  the commit is the one
    // get the hash and commiter of the commit
    let repo_path = tmp_dir.path();
    let repo = Repository::open(repo_path)?;
    let branch = repo.find_branch("christmas", BranchType::Local)?;
    let base_commit = branch.get().peel_to_commit()?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(base_commit.id())?;
    let author_and_commit = revwalk
        .find_map(|commit_oid| {
            let commit_oid = commit_oid.ok()?;
            let commit = repo.find_commit(commit_oid).ok()?;
            info!(commit = ?commit, "At commit");
            let commit_tree = commit.tree().ok()?;
            let mut conditition = false;
            let _ = commit_tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
                let name = entry.name();
                if name.is_none() {
                    return TreeWalkResult::Ok;
                }
                let name = name.unwrap();
                if name != "santa.txt" {
                    return TreeWalkResult::Ok;
                }
                let Ok(object) = entry.to_object(&repo) else {
                    return TreeWalkResult::Ok;
                };
                let Some(blob) = object.as_blob() else {
                    return TreeWalkResult::Ok;
                };
                let Ok(contents) = from_utf8(blob.content()) else {
                    return TreeWalkResult::Ok;
                };
                if contents.contains("COOKIE") {
                    info!("We got em");
                    conditition = true;
                    TreeWalkResult::Abort
                } else {
                    info!("No Cookie here");
                    TreeWalkResult::Ok
                }
            });
            if conditition {
                let author = commit.author();
                let name = author.name()?;
                let hash = commit.id();
                Some(format!("{name} {hash}"))
            } else {
                None
            }
        })
        .ok_or_eyre("Couldn't find the commit")?;
    Ok(author_and_commit)
}
