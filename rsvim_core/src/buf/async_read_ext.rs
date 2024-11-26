//! Async read extensions for vim buffer.

use crate::buf::{BufferArc, BufferStatus};
use crate::envar;
use crate::evloop::msg::{ReadBytes, WorkerToMasterMessage};
use crate::res::{TheBufferErr, TheBufferResult};
use crate::{rlock, wlock};

use ropey::{Rope, RopeBuilder};
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

fn into_rope(buf: &[u8], bufsize: usize) -> Rope {
  let bufstr = into_str(buf, bufsize);
  let mut block = RopeBuilder::new();
  block.append(&bufstr.to_owned());
  block.finish()
}

fn into_str(buf: &[u8], bufsize: usize) -> String {
  String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
}

/// Bind buffer to a file.
///
/// If the buffer is initialized, it will immediately start loading
/// If the buffer is already associated with file, it will be renamed to new filename (only if the
/// file not exists).
///
/// Returns
///
/// - It returns the reading bytes after file loaded (if successful).
/// - Otherwise it returns [`TheBufferErr`](crate::res::TheBufferErr) to indicate some errors.
async fn buffer_bind_async(buf: BufferArc, filename: &str) -> TheBufferResult<usize> {
  let buf_status = rlock!(buf).status();
  let buf_associated = rlock!(buf).filename().clone();

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
                debug!("Read {} bytes: {:?}", n, into_str(&iobuf, n));

                // Load into buffer.
                let mut wbuf = wlock!(buf);
                wbuf.append(into_rope(&iobuf, n));

                // After read each block, immediately notify main thread so UI tree can render
                // it on terminal.
                debug!("Notify master after each block read");
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
            wbuf.set_status(BufferStatus::CHANGED);
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
