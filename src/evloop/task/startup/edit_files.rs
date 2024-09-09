//! Edit files on start up.

use futures::future::{BoxFuture, Future};
use ropey::{Rope, RopeBuilder};
use std::pin::Pin;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::buf::{Buffer, Buffers, BuffersArc};
use crate::evloop::task::{TaskResult, TaskableDataAccess};
use crate::glovar;

fn into_repo(buf: &[u8]) -> Rope {
  let buf1: &[u8] = buf;
  let buf1str: String = String::from_utf8_lossy(buf1).into_owned();

  let mut block = RopeBuilder::new();
  block.append(&buf1str.to_owned());
  block.finish()
}

/// Edit files
pub async fn edit_files(data_access: TaskableDataAccess, files: Vec<String>) -> TaskResult {
  let rbuf_size = 4096_usize;
  let buffers = data_access.buffers.clone();

  let mut first_buffer_created = false;
  let mut first_block_read = true;

  for (i, file) in files.iter().enumerate() {
    debug!("Read the {} input file: {:?}", i, file);
    match fs::File::open(file).await {
      Ok(mut file) => {
        let mut buf: Vec<u8> = vec![0_u8; rbuf_size];
        let mut builder = Rope::new();

        debug!("Read buffer bytes size: {}", buf.len());
        loop {
          match file.read_buf(&mut buf).await {
            Ok(n) => {
              debug!("Read {} bytes", n);

              builder.append(into_repo(&buf));
              if first_block_read {
                first_block_read = false;
                // After read the first block, immediately yield to the main thread so UI tree can
                // render it on terminal.
                tokio::task::yield_now().await;
              }

              if n == 0 {
                // Finish reading
                if !first_buffer_created {
                  // For the default (first) buffer, append it
                  buffers
                    .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                    .unwrap()
                    .first_key_value()
                    .unwrap()
                    .1
                    .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                    .unwrap()
                    .rope_mut()
                    .append(builder);
                  first_buffer_created = true;
                } else {
                  // For others, insert new buffers.
                  buffers
                    .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                    .unwrap()
                    .insert(Buffer::to_arc(Buffer::from(builder)));
                }

                // println!("Read file {:?} into buffer", input_file);
                break;
              }
            }
            Err(e) => {
              // Unexpected error
              let msg = format!("Failed to read file {:?} with error {:?}", file, e);
              error!("{msg}");
              return Err(msg);
            }
          }
        }
      }
      Err(e) => {
        let msg = format!("Failed to open file {:?} with error {:?}", file, e);
        error!("{msg}");
        return Err(msg);
      }
    }
  }

  Ok(())
}
