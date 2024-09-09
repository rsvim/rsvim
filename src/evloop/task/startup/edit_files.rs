//! Edit files on start up.

use futures::future::{BoxFuture, Future};
use ropey::RopeBuilder;
use std::pin::Pin;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::buf::{Buffer, Buffers, BuffersArc};
use crate::evloop::task::{TaskResult, TaskableDataAccess};
use crate::glovar;

/// Edit files
pub async fn edit_files(data_access: TaskableDataAccess, files: Vec<String>) -> TaskResult {
  let rbuf_size = 4096_usize;
  let buffers = data_access.buffers.clone();

  let default_buffer = {
    buffers
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .first_key_value()
      .unwrap()
      .1
  };

  for (i, file) in files.iter().enumerate() {
    debug!("Read the {} input file: {:?}", i, file);
    match fs::File::open(file).await {
      Ok(mut file) => {
        let mut builder = RopeBuilder::new();

        let mut rbuf: Vec<u8> = vec![0_u8; rbuf_size];
        debug!("Read buffer bytes size: {}", rbuf.len());
        loop {
          match file.read_buf(&mut rbuf).await {
            Ok(n) => {
              debug!("Read {} bytes", n);
              let rbuf1: &[u8] = &rbuf;
              let rbuf_str: String = String::from_utf8_lossy(rbuf1).into_owned();

              builder.append(&rbuf_str.to_owned());
              if n == 0 {
                // Finish reading, create new buffer
                let buffer = Buffer::from(builder);
                buffers
                  .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                  .unwrap()
                  .insert(Buffer::to_arc(buffer));

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
