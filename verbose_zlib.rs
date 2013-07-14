extern mod comprsr_zlib
  (vers = "0.0.1", author = "github.com/honzasp");

use std::os;
use std::io;
use std::path;
use std::from_str;
use std::cmp;
use std::str;

use zlib = comprsr_zlib::zlib;

fn main() {
  let args = os::args();

  let (file_name, chunk_size): (~str, uint) = match args.tail() {
    [ref file_name] => (file_name.clone(), 1024),
    [ref file_name, ref chunk_size] =>
      match from_str::FromStr::from_str(*chunk_size) {
        Some(chunk_size) => (file_name.clone(), chunk_size),
        None => {
          error(fmt!("chunk_size was not a number\n"));
          return;
        }
    },
    _ => {
      error(fmt!("Args: input_file [chunk_size]\n"));
      return;
    },
  };

  show(fmt!("input %s, chunk size %u\n", file_name, chunk_size));

  show("reading file ...");
  let compr_data = match io::read_whole_file(&path::PosixPath(file_name)) {
    Ok(bytes) => bytes,
    Err(err) => {
      error(fmt!("Error reading %s: %s\n", file_name, err));
      return;
    },
  };
  show(fmt!(" ok, %u b\n", compr_data.len()));

  decode(compr_data, chunk_size);
}

fn decode(compr_data: &[u8], chunk_size: uint) {
  let mut decoder = zlib::decoder::Decoder::new();
  let mut chunk_begin = 0;
  let mut whole_output = ~[];

  while chunk_begin < compr_data.len() {
    let chunk_end = cmp::min(chunk_begin + chunk_size, compr_data.len());
    let is_last = chunk_end == compr_data.len();
    let slice = compr_data.slice(chunk_begin, chunk_end);

    show(fmt!("- %6u ... %6u\n", chunk_begin, chunk_end));
    let empty_buf: ~[u8] = ~[];
    let (result, output) = decoder.input(slice, empty_buf);

    let row_len = 20;
    let mut i = 0;
    while i < output.len() {
      show("    ");
      let j = cmp::min(i + row_len, output.len());
      let slice = output.slice(i, j);
      i = j;

      for slice.iter().advance |&byte| {
        show(fmt!("%02x ", byte as uint));
      }

      show("  \"");
      for slice.iter().advance |&byte| {
        show(str::from_byte(byte).escape_default());
      }
      show("\"\n");
    }

    match result {
      Left(new_decoder) => {
        whole_output.push_all(output);
        show(fmt!("  > out %6u b, total %6u b\n", output.len(), whole_output.len()));
        decoder = new_decoder;
      },
      Right((outcome, rest)) => {
        show(fmt!(" >  %6u bytes left: ", rest.len()));
        match outcome {
          Ok(()) if is_last => {
            show(fmt!("ok\n"));
          },
          Ok(()) => {
            show(fmt!("! finished with %6u b left\n", rest.len()));
          },
          Err(err) => {
            show(fmt!("! err: %s\n", err.to_str()));
          },
        };
        return;
      },
    }

    chunk_begin = chunk_begin + chunk_size;
  }

  show(fmt!("! the decoder did not finish\n"));
}


fn error(msg: &str) {
  io::stderr().write_str(msg);
  os::set_exit_status(1);
}

fn show(msg: &str) {
  io::stdout().write_str(msg);
  io::stdout().flush();
}
