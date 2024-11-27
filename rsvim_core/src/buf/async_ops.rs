//! Async operations for vim buffer.
//!
//! This module abstract common async read/write operations on Vim buffer, and provide a unified
//! interface for upper level logic. Most operations will lock the buffer to read/write its
//! internal data, but overall they don't block the editor main loop.

use crate::buf::opt::FileEncoding;
use crate::buf::{BufferArc, BufferStatus};
use crate::envar;
use crate::evloop::msg::{ReadBytes, WorkerToMasterMessage};
use crate::res::{IoError, IoResult};
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
/// - Otherwise it returns [`TheBufferErr`](crate::res::TheBufferErr) to indicate some errors.
async fn open_file_async(buf: BufferArc, filename: &str) -> IoResult<usize> {
  let (buf_status, buf_associated, buf_fencoding) = {
    let rbuf = rlock!(buf);
    (
      rbuf.status(),
      rbuf.filename().clone(),
      rbuf.options().file_encoding(),
    )
  };

  match buf_associated {
    Some(associated) => {
      // Associated
      assert!(
        buf_status == BufferStatus::SYNCED
          || buf_status == BufferStatus::LOADING
          || buf_status == BufferStatus::SAVING
          || buf_status == BufferStatus::CHANGED
      );
      if associated == filename {
        // The same file
        return Err(TheBufferErr::BufferAlreadyBinded(associated.to_string()));
      }

      let file_exists = match tokio::fs::try_exists(filename).await {
        Ok(exists) => exists,
        Err(e) => return Err(TheBufferErr::IoErr(e)),
      };

      if file_exists {
        // File already exists
        return Err(TheBufferErr::FileAlreadyExists(filename.to_string()));
      } else {
        // File not exists
        // Rename buffer, or say, bind to new filename, change status to `CHANGED`.
        let mut wbuf = wlock!(buf);
        wbuf.set_filename(Some(filename.to_string()));
        wbuf.set_status(BufferStatus::CHANGED);
        return Ok(0_usize);
      }
    }
    None => {
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

          let mut total_read_bytes = 0_usize;
          let mut iobuf: Vec<u8> = vec![0_u8; envar::IO_BUF_SIZE()];

          loop {
            match fp.read(&mut iobuf).await {
              Ok(n) => {
                debug!("Read {} bytes: {:?}", n, into_str(&iobuf, n, buf_fencoding));

                // Load into buffer, and notify master thread so UI tree could update renderings.
                let mut wbuf = wlock!(buf);
                wbuf.append(into_rope(&iobuf, n, buf_fencoding));
                wbuf
                  .worker_send_to_master()
                  .send(WorkerToMasterMessage::ReadBytes(ReadBytes::new(n)))
                  .await
                  .unwrap();

                if n == 0 {
                  // Finish reading, exit loop
                  break;
                }

                total_read_bytes += n;
              }
              Err(e) => {
                // Unexpected error
                error!("Error reading file: {:?}:{:?}", e.kind(), e.to_string());
                return Err(TheBufferErr::IoErr(e));
              }
            }
          }

          {
            let mut wbuf = wlock!(buf);
            wbuf.set_status(BufferStatus::SYNCED);
          }
          return Ok(total_read_bytes);
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
      };
    }
  }
}
