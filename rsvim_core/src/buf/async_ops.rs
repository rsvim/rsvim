//! Async operations for vim buffer.
//!
//! This module abstract common async read/write operations on Vim buffer, and provide a unified
//! interface for upper level logic. Most operations will lock the buffer to read/write its
//! internal data, but overall they don't block the editor main loop.

use crate::buf::opt::FileEncoding;
use crate::buf::{BufferArc, BufferStatus};
use crate::envar;
use crate::evloop::msg::{BufferLoadedBytes, WorkerToMasterMessage};
use crate::res::IoResult;
use crate::{rlock, wlock};

use ropey::{Rope, RopeBuilder};
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

fn into_str(buf: &[u8], bufsize: usize, fencoding: FileEncoding) -> String {
  match fencoding {
    FileEncoding::Utf8 => String::from_utf8_lossy(&buf[0..bufsize]).into_owned(),
  }
}

fn into_rope(buf: &[u8], bufsize: usize, fencoding: FileEncoding) -> Rope {
  let bufstr = into_str(buf, bufsize, fencoding);
  let mut block = RopeBuilder::new();
  block.append(&bufstr.to_owned());
  block.finish()
}

/// Open file with buffer.
///
/// Also see <https://vimhelp.org/editing.txt.html#%3Aedit>.
///
/// If the file exists, it will immediately start loading.
///
/// Returns
///
/// - It returns the reading bytes after file loaded (if successful).
/// - Otherwise it returns [`IoError`](crate::res::IoError) to indicate some errors.
async fn open_file_async(buf: BufferArc, filename: &str) -> IoResult<usize> {
  let (buf_status, buf_not_associated, buf_options) = {
    let rbuf = rlock!(buf);
    (
      rbuf.status(),
      rbuf.filename().is_none(),
      rbuf.options().clone(),
    )
  };

  // Buffer must be detached.
  assert!(buf_not_associated);

  // Detached
  assert!(buf_status == BufferStatus::INIT || buf_status == BufferStatus::CHANGED);

  match tokio::fs::File::open(filename).await {
    Ok(mut fp) => {
      // Rename buffer, or say, bind to new filename, start `LOADING`.
      {
        let mut wbuf = wlock!(buf);
        wbuf.set_filename(Some(filename.to_string()));
        wbuf.set_status(BufferStatus::LOADING);
      }

      match fp.metadata().await {
        Ok(metadata) => {
          wlock!(buf).set_metadata(Some(metadata));
        }
        Err(e) => {
          // Unexpected error
          error!(
            "Error fetching metainfo from file {:?}:{:?}",
            e.kind(),
            e.to_string()
          );
          return Err(e);
        }
      }

      let mut read_bytes = 0_usize;
      let mut read_buf: Vec<u8> = vec![0_u8; envar::IO_BUF_SIZE()];

      loop {
        match fp.read(&mut read_buf).await {
          Ok(n) => {
            debug!(
              "Read {} bytes: {:?}",
              n,
              into_str(&read_buf, n, buf_options.file_encoding())
            );

            // Load into buffer, and notify master thread so UI tree could update renderings.
            let mut wbuf = wlock!(buf);
            wbuf.append(into_rope(&read_buf, n, buf_options.file_encoding()));
            wbuf
              .worker_send_to_master()
              .send(WorkerToMasterMessage::BufferLoadedBytes(
                BufferLoadedBytes::new(n),
              ))
              .await
              .unwrap();

            if n == 0 {
              // Finish reading, exit loop
              break;
            }

            read_bytes += n;
          }
          Err(e) => {
            // Unexpected error
            error!("Error reading file {:?}:{:?}", e.kind(), e.to_string());
            return Err(e);
          }
        }
      }

      {
        let mut wbuf = wlock!(buf);
        wbuf.set_status(BufferStatus::SYNCED);
      }
      return Ok(read_bytes);
    }
    Err(e) => match e.kind() {
      std::io::ErrorKind::NotFound => {
        // File not found
        // Rename buffer, or say, bind to new filename, change status to `CHANGED`.
        let mut wbuf = wlock!(buf);
        wbuf.set_filename(Some(filename.to_string()));
        wbuf.set_status(BufferStatus::SYNCED);
        return Ok(0_usize);
      }
      _ => {
        error!("Error opening file: {:?}:{:?}", e.kind(), e.to_string());
        return Err(TheBufferErr::IoErr(e));
      }
    },
  }
}
