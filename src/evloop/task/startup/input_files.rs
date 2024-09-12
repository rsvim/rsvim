//! Edit input files on start up.

use futures::future::{BoxFuture, Future};
use ropey::{Rope, RopeBuilder};
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::buf::{Buffer, BufferArc, Buffers, BuffersArc};
use crate::evloop::message::{Dummy, Notify};
use crate::evloop::task::{TaskResult, TaskableDataAccess};
use crate::glovar;
use crate::result::ErrorCode;

fn buf_size() -> usize {
  static VALUE: OnceLock<usize> = OnceLock::new();

  *VALUE.get_or_init(|| 8192_usize)
}

fn into_str(buf: &[u8], bufsize: usize) -> String {
  String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
}

fn into_repo(buf: &[u8], bufsize: usize) -> Rope {
  let bufstr = into_str(buf, bufsize);
  let mut block = RopeBuilder::new();
  block.append(&bufstr.to_owned());
  block.finish()
}

/// Edit default input file for the default buffer, i.e. the empty buffer created along with
/// default window.
pub async fn edit_default_file(data_access: TaskableDataAccess, file_name: String) -> TaskResult {
  let buffers = data_access.buffers.clone();
  let worker_sender = data_access.worker_sender;

  debug!("Read the default input file: {:?}", file_name.as_str());
  match fs::File::open(file_name.as_str()).await {
    Ok(mut fp) => {
      let mut buf: Vec<u8> = vec![0_u8; buf_size()];
      loop {
        match fp.read(&mut buf).await {
          Ok(n) => {
            debug!("Read {} bytes: {:?}", n, into_str(&buf, n));

            // For the first buffer, append to the **default** buffer.
            buffers
              .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
              .unwrap()
              .first_key_value()
              .unwrap()
              .1
              .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
              .unwrap()
              .rope_mut()
              .append(into_repo(&buf, n));

            // After read each block, immediately notify main thread so UI tree can render it on
            // terminal.
            debug!("Notify master after each block read");
            worker_sender.send(Notify::Dummy(Dummy::default())).unwrap();

            if n == 0 {
              // Finish reading, exit loop
              break;
            }
          }
          Err(e) => {
            // Unexpected error
            let msg = format!(
              "Failed to read default input file {:?} with error {:?}",
              file_name.as_str(),
              e
            );
            error!("{msg}");
            return Err(ErrorCode::Message(msg));
          }
        }
      }
    }
    Err(e) => {
      let msg = format!(
        "Failed to open default input file {:?} with error {:?}",
        file_name.as_str(),
        e
      );
      error!("{msg}");
      return Err(ErrorCode::Message(msg));
    }
  }

  Ok(())
}

/// Edit other input files for newly created buffers.
pub async fn edit_other_files(
  data_access: TaskableDataAccess,
  file_names: Vec<String>,
) -> TaskResult {
  let buffers = data_access.buffers.clone();
  let worker_sender = data_access.worker_sender;

  for (i, file_name) in file_names.iter().enumerate() {
    debug!("Read the {} input file: {:?}", i, file_name.as_str());
    match fs::File::open(file_name.as_str()).await {
      Ok(mut fp) => {
        let mut buf: Vec<u8> = vec![0_u8; buf_size()];

        // Create new buffer
        let buffer = Buffer::to_arc(Buffer::from(Rope::new()));
        buffers
          .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
          .unwrap()
          .insert(buffer.clone());

        loop {
          match fp.read_buf(&mut buf).await {
            Ok(n) => {
              debug!("Read {} bytes: {:?}", n, into_str(&buf, n));

              buffer
                .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                .unwrap()
                .rope_mut()
                .append(into_repo(&buf, n));

              // After read each block, immediately notify main thread so UI tree can render it on
              // terminal.
              debug!("Notify master after each block read");
              worker_sender.send(Notify::Dummy(Dummy::default())).unwrap();

              if n == 0 {
                // Finish reading, exit loop
                break;
              }
            }
            Err(e) => {
              // Unexpected error
              let msg = format!(
                "Failed to read the {:?} other file {:?} with error {:?}",
                i,
                file_name.as_str(),
                e
              );
              error!("{msg}");
              return Err(ErrorCode::Message(msg));
            }
          }
        }
      }
      Err(e) => {
        let msg = format!(
          "Failed to open the {:?} other file {:?} with error {:?}",
          i,
          file_name.as_str(),
          e
        );
        error!("{msg}");
        return Err(ErrorCode::Message(msg));
      }
    }
  }

  Ok(())
}
