extern mod comprsr_zlib;

use std::io;
use std::os;

mod zlib { pub use comprsr_zlib::zlib::*; }

fn main() {
  let test_root = match os::self_exe_path() {
      Some(self_path) => self_path.dir_path(),
      None => fail!(~"os::self_exe_path() returned None"),
    };

  let samples_dir = test_root.push("samples");
  let zlib_dir = test_root.push("zlib");

  show("reading samples ");
  let samples: ~[(~str, ~[u8])] = os::list_dir_path(&samples_dir)
    .map(|&sample_path| {
      let name = sample_path.filename().unwrap();
      let data = match io::read_whole_file(sample_path) {
          Ok(data) => data,
          Err(err) => fail!(fmt!("reading %? failed: %?", sample_path, err)),
        };
      show(".");
      (name, data)
    });
  show(" ok\n");

  for os::list_dir_path(&zlib_dir).each |&zlib_sub| {
    if os::path_is_dir(zlib_sub) {
      test_subdir(zlib_sub, samples);
    }
  }
}

fn test_subdir(dir: &Path, samples: &[(~str, ~[u8])]) {
  show(fmt!("testing %s\n", dir.to_str()));
  for samples.each |&(name, expected_data)| {
    let compr_file = dir.push(fmt!("%s.zlib", name));
    let compr_reader = match io::file_reader(&compr_file) {
        Ok(reader) => reader,
        Err(err) => fail!(fmt!("opening %s failed: %s",
          compr_file.to_str(), err.to_str())),
      };

    show(fmt!("  %?: ", name));
    match test_zlib(expected_data, compr_reader) {
      Ok(()) => show("ok"),
      Err(err) => show(fmt!("err: %?", err)),
    };
    show("\n");
  }
}

fn test_zlib(expected_data: &[u8], compr_reader: @io::Reader)
  -> Result<(), ~str> 
{
  let mut decoder = zlib::decoder::Decoder::new(~~[]);

  while !compr_reader.eof() {
    let mut buf = ~[0, ..256];
    let read = compr_reader.read(buf, buf.len());
    match decoder.input(buf.slice(0, read)) {
      zlib::decoder::ConsumedRes => { },
      zlib::decoder::FinishedRes(rest) =>
        if !rest.is_empty() {
          return Err(fmt!("decoder finished before end of input (%? bytes)",
            rest.len()))
        },
      zlib::decoder::ErrorRes(err, _rest) =>
        return Err(fmt!("decoder returned error: %?", err)),
    }
  }

  if decoder.has_finished() {
    let result = *decoder.finish();
    if expected_data == result {
      Ok(())
    } else {
      Err(fmt!("decoder returned wrong result!!!"))
    }
  } else {
    Err(fmt!("decoder has not finished before end of input"))
  }
}

fn show(msg: &str) {
  io::stdout().write(msg.as_bytes());
  io::stdout().flush();
}
