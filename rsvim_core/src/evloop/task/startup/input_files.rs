//! Edit input files on start up.

use crate::envar;
use crate::evloop::msg::{BufferLoadedBytes, WorkerToMasterMessage};
use crate::evloop::task::TaskableDataAccess;
use crate::res::AnyResult;
use crate::{rlock, wlock};

use ropey::{Rope, RopeBuilder};
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

fn into_str(buf: &[u8], bufsize: usize) -> String {
  String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
}

fn into_rope(buf: &[u8], bufsize: usize) -> Rope {
  let bufstr = into_str(buf, bufsize);
  let mut block = RopeBuilder::new();
  block.append(&bufstr.to_owned());
  block.finish()
}

/// Edit default input file for the default buffer, i.e. the empty buffer created along with
/// default window.
pub async fn edit_default_file(
  data_access: TaskableDataAccess,
  file_name: String,
) -> AnyResult<()> {
  let buffers = data_access.buffers.clone();
  let worker_send_to_master = data_access.worker_send_to_master;

  debug!("Read the default input file: {:?}", file_name.as_str());
  match fs::File::open(file_name.as_str()).await {
    Ok(mut fp) => {
      let mut buf: Vec<u8> = vec![0_u8; envar::IO_BUF_SIZE()];
      loop {
        match fp.read(&mut buf).await {
          Ok(n) => {
            debug!("Read {} bytes: {:?}", n, into_str(&buf, n));

            // For the first buffer, append to the **default** buffer.
            rlock!(buffers)
              .first_key_value()
              .unwrap()
              .1
              .try_write_for(envar::MUTEX_TIMEOUT())
              .unwrap()
              .append(into_rope(&buf, n));

            // After read each block, immediately notify main thread so UI tree can render it on
            // terminal.
            debug!("Notify master after each block read");
            worker_send_to_master
              .send(WorkerToMasterMessage::BufferLoadedBytes(
                BufferLoadedBytes::new(n),
              ))
              .await
              .unwrap();

            if n == 0 {
              // Finish reading, exit loop
              break;
            }
          }
          Err(e) => {
            // Unexpected error
            let e = format!(
              "Failed to read default input file {:?} with error {:?}",
              file_name.as_str(),
              e
            );
            error!("{e}");
            anyhow::bail!(e);
          }
        }
      }
    }
    Err(e) => {
      let e = format!(
        "Failed to open default input file {:?} with error {:?}",
        file_name.as_str(),
        e
      );
      error!("{e}");
      anyhow::bail!(e);
    }
  }

  Ok(())
}

/// Edit other input files for newly created buffers.
pub async fn edit_other_files(
  data_access: TaskableDataAccess,
  file_names: Vec<String>,
) -> AnyResult<()> {
  let buffers = data_access.buffers.clone();
  let worker_sender = data_access.worker_send_to_master;

  for (i, file_name) in file_names.iter().enumerate() {
    debug!("Read the {} input file: {:?}", i, file_name.as_str());
    match fs::File::open(file_name.as_str()).await {
      Ok(mut fp) => {
        let mut buf: Vec<u8> = vec![0_u8; envar::IO_BUF_SIZE()];

        // Create new buffer
        let buf_id = wlock!(buffers).new_buffer_with_edit_file();
        let buffer = rlock!(buffers).get(&buf_id).unwrap().clone();

        loop {
          match fp.read_buf(&mut buf).await {
            Ok(n) => {
              debug!("Read {} bytes: {:?}", n, into_str(&buf, n));

              wlock!(buffer).append(into_rope(&buf, n));

              // After read each block, immediately notify main thread so UI tree can render it on
              // terminal.
              debug!("Notify master after each block read");
              worker_sender
                .send(WorkerToMasterMessage::BufferLoadedBytes(
                  BufferLoadedBytes::new(n),
                ))
                .await
                .unwrap();

              if n == 0 {
                // Finish reading, exit loop
                break;
              }
            }
            Err(e) => {
              // Unexpected error
              let e = format!(
                "Failed to read the {:?} other file {:?} with error {:?}",
                i,
                file_name.as_str(),
                e
              );
              error!("{e}");
              anyhow::bail!(e);
            }
          }
        }
      }
      Err(e) => {
        let e = format!(
          "Failed to open the {:?} other file {:?} with error {:?}",
          i,
          file_name.as_str(),
          e
        );
        error!("{e}");
        anyhow::bail!(e);
      }
    }
  }

  Ok(())
}
