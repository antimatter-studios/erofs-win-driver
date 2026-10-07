//! A directory the reader cannot list is an error, never an empty folder.
//!
//! The WinFsp `read_directory` callback listed a directory with
//! `if let Ok(children) = ...`, so a directory whose data could not be
//! read came back to Windows as a folder with nothing in it, and a child
//! whose inode could not be read silently dropped out of the listing. An
//! empty folder is the one answer a user cannot tell from a correct one.
//!
//! The `Rebuild` dismount policy walked the tree the same way, and it
//! writes what it saw: a subtree it could not read was left out of the
//! rebuilt image, and the rebuild reported success.
//!
//! These read an image through a device that fails on demand, so the
//! failure is a real read error from the real read path.

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use erofs_win_driver::mount::{ntstatus, ntstatus_for, rebuild_image, underlay_children};
use erofs_win_driver::overlay::Overlay;
use fs_core::BlockRead;
use fs_erofs::mkfs::{build_image, Node, NodeMeta, DEFAULT_DIR_MODE, DEFAULT_FILE_MODE};
use fs_erofs::{Filesystem, Inode};

/// How many files `/manyentries` holds.
const ENTRIES: usize = 200;

/// An image in memory, read through a switch: once `fail_all` is set,
/// or for any read overlapping `fail_range`, the read fails as the
/// device would.
struct FailingDevice {
    bytes: Vec<u8>,
    fail_all: AtomicBool,
    fail_range: Mutex<Option<Range<u64>>>,
}

impl BlockRead for FailingDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> fs_core::Result<()> {
        let end = offset + buf.len() as u64;
        let in_range = self
            .fail_range
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|r| offset < r.end && r.start < end);
        if self.fail_all.load(Ordering::SeqCst) || in_range {
            return Err(fs_core::Error::Io(std::io::Error::other(
                "injected read failure",
            )));
        }
        let (start, end) = (offset as usize, end as usize);
        if end > self.bytes.len() {
            return Err(fs_core::Error::Io(std::io::Error::from(
                std::io::ErrorKind::UnexpectedEof,
            )));
        }
        buf.copy_from_slice(&self.bytes[start..end]);
        Ok(())
    }

    fn size_bytes(&self) -> u64 {
        self.bytes.len() as u64
    }
}

fn file(data: &[u8]) -> Node {
    Node::File {
        mode: DEFAULT_FILE_MODE,
        data: data.to_vec(),
        meta: NodeMeta::default(),
        xattrs: Vec::new(),
    }
}

fn dir(entries: BTreeMap<String, Node>) -> Node {
    Node::Dir {
        mode: DEFAULT_DIR_MODE,
        entries,
        meta: NodeMeta::default(),
        xattrs: Vec::new(),
    }
}

/// `/small.txt`, and `/manyentries` holding `ENTRIES` small files: more
/// than one block of directory entries and of inodes.
fn image() -> Vec<u8> {
    let many = (1..=ENTRIES)
        .map(|i| (format!("entry-{i}.txt"), file(format!("e{i}").as_bytes())))
        .collect();
    let mut root = BTreeMap::new();
    root.insert("small.txt".to_string(), file(b"hello erofs"));
    root.insert("manyentries".to_string(), dir(many));
    build_image(dir(root), 12).expect("build the image")
}

/// `bytes` through a `FailingDevice` that is not failing yet.
fn device(bytes: Vec<u8>) -> Arc<FailingDevice> {
    Arc::new(FailingDevice {
        bytes,
        fail_all: AtomicBool::new(false),
        fail_range: Mutex::new(None),
    })
}

/// The image through a `FailingDevice`, with no block cache: a cached
/// block would answer a read the test means to fail.
fn open() -> (Arc<FailingDevice>, Filesystem) {
    let dev = device(image());
    let fs =
        Filesystem::open_with_cache(dev.clone() as Arc<dyn BlockRead>, 0).expect("open the image");
    (dev, fs)
}

/// Fail the read of one child of `/manyentries`: the one whose inode is
/// furthest into the image, so the directory's own inode and entries
/// stay readable. Returns the directory, after checking it still lists.
fn fail_one_child(dev: &FailingDevice, fs: &Filesystem) -> Inode {
    let dir = fs
        .lookup_path("/manyentries")
        .expect("look up /manyentries");
    let victim = fs
        .read_dir(&dir)
        .expect("list /manyentries")
        .iter()
        .filter(|e| e.name != b"." && e.name != b"..")
        .map(|e| Inode::iloc(fs.superblock(), e.nid))
        .max()
        .expect("a child");
    *dev.fail_range.lock().unwrap() = Some(victim..victim + 1);
    let dir = fs
        .lookup_path("/manyentries")
        .expect("the directory itself is still readable");
    fs.read_dir(&dir)
        .expect("the directory's entries are still readable");
    dir
}

/// With nothing failing, every entry the image holds is listed.
#[test]
fn a_readable_directory_lists_every_entry() {
    let (_dev, fs) = open();
    let dir = fs
        .lookup_path("/manyentries")
        .expect("look up /manyentries");
    let listed = underlay_children(&fs, &Overlay::new(), &dir, "/manyentries")
        .expect("a readable directory lists");
    assert_eq!(listed.len(), ENTRIES, "every entry the image holds");
}

/// A directory whose contents cannot be read is an I/O error, not an
/// empty listing.
#[test]
fn an_unreadable_directory_is_an_io_error_not_an_empty_folder() {
    let (dev, fs) = open();
    let dir = fs
        .lookup_path("/manyentries")
        .expect("look up /manyentries");
    dev.fail_all.store(true, Ordering::SeqCst);

    let err = match underlay_children(&fs, &Overlay::new(), &dir, "/manyentries") {
        Ok(entries) => panic!(
            "an unreadable directory was listed as {} entries instead of failing",
            entries.len()
        ),
        Err(e) => e,
    };
    assert_eq!(
        ntstatus_for(&err),
        ntstatus::IO_DEVICE_ERROR,
        "a read failure reaches Windows as STATUS_IO_DEVICE_ERROR, got {err}"
    );
}

/// A child whose inode cannot be read fails the listing rather than
/// vanishing from it.
#[test]
fn an_unreadable_child_fails_the_listing_rather_than_vanishing() {
    let (dev, fs) = open();
    let dir = fail_one_child(&dev, &fs);

    let err = match underlay_children(&fs, &Overlay::new(), &dir, "/manyentries") {
        Ok(listed) => panic!(
            "a child that could not be read was dropped: {} of {ENTRIES} entries listed, no error",
            listed.len()
        ),
        Err(e) => e,
    };
    assert_eq!(
        ntstatus_for(&err),
        ntstatus::IO_DEVICE_ERROR,
        "a read failure reaches Windows as STATUS_IO_DEVICE_ERROR, got {err}"
    );
}

/// A rebuild that cannot read part of the underlay fails, rather than
/// writing an image without it.
#[test]
fn a_rebuild_that_cannot_read_the_underlay_fails_rather_than_dropping_it() {
    let (dev, fs) = open();
    fail_one_child(&dev, &fs);
    let out = tempfile::NamedTempFile::new().expect("tempfile");

    if rebuild_image(&fs, &Overlay::new(), out.path()).is_ok() {
        let rebuilt = std::fs::read(out.path()).expect("read the rebuilt image");
        let rebuilt = Filesystem::open(device(rebuilt)).expect("open the rebuilt image");
        let dir = rebuilt
            .lookup_path("/manyentries")
            .expect("look up /manyentries in the rebuilt image");
        let kept = rebuilt
            .read_dir(&dir)
            .expect("list the rebuilt /manyentries")
            .iter()
            .filter(|e| e.name != b"." && e.name != b"..")
            .count();
        panic!("the rebuild reported success with {kept} of {ENTRIES} entries in /manyentries");
    }
}
