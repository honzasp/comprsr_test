extern mod extra;
extern mod comprsr_zlib
  (vers = "0.0.1", author = "github.com/honzasp");

use std::io;
use std::os;
use extra::time;

mod zlib { pub use comprsr_zlib::zlib::*; }

fn main() {
  let test_root = match os::self_exe_path() {
      Some(self_path) => self_path,
      None => fail!(~"os::self_exe_path() returned None"),
    };

  let samples_dir = &test_root.push("samples");
  let zlib_dir = &test_root.push("zlib");

  show("reading samples ");
  let samples: ~[(~str, ~[u8])] = os::list_dir_path(samples_dir)
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

  let zlib_dir_paths = os::list_dir_path(zlib_dir);
  for zlib_dir_paths.iter().advance |&zlib_sub| {
    if os::path_is_dir(zlib_sub) {
      process_subdir(zlib_sub, samples);
    }
  };
}

#[cfg(tester)]
fn process_subdir(dir: &Path, samples: &[(~str, ~[u8])]) {
  test_subdir(dir, samples);
}

#[cfg(benchmarker)]
fn process_subdir(dir: &Path, samples: &[(~str, ~[u8])]) {
  bench_subdir(dir, samples.map(|s| s.first()));
}

fn test_subdir(dir: &Path, samples: &[(~str, ~[u8])]) {
  show(fmt!("testing %s\n", dir.to_str()));
  for samples.iter().advance |&(name, expected_data)| {
    show(fmt!("  %s", name));

    let compr_file = dir.push(fmt!("%s.zlib", name));
    let compr_reader = match io::file_reader(&compr_file) {
        Ok(reader) => reader,
        Err(err) => fail!(fmt!("opening %s failed: %s",
          compr_file.to_str(), err.to_str())),
      };

    show(": ");
    match test_zlib(expected_data, compr_reader) {
      Ok(()) => show("ok"),
      Err(err) => show(fmt!("err: %s", err.to_str())),
    };
    show("\n");
  }
}

fn bench_subdir(dir: &Path, samples: &[~str]) {
  let mut total_time: f64 = 0.0;

  show(fmt!("benchmarking %s\n", dir.to_str()));
  for samples.iter().advance |&name| {
    show(fmt!("  %s", name));

    let compr_file = dir.push(fmt!("%s.zlib", name));
    let compr_data = match io::read_whole_file(&compr_file) {
        Ok(data) => data,
        Err(err) => fail!(fmt!("reading %s failed: %s",
          compr_file.to_str(), err.to_str())),
      };

    show(": ");
    do io::with_bytes_reader(compr_data) |reader| {
      match bench_zlib(reader) {
        Ok(time) => {
          for (40 - name.len()).times {
            show(" "); 
          };
          show(fmt!("%s s", time.to_str()));
          total_time = total_time + time;
        },
        Err(err) => show(fmt!("err: %s", err.to_str())),
      }
    };
    show("\n");
  }

  show(     "  -----------\n");
  show(fmt!("              %s s\n", total_time.to_str()));
  show("\n");
}

fn run_zlib(compr_reader: @io::Reader) -> Result<~[u8], ~str> {
  let mut decoder = zlib::decoder::Decoder::new(~~[]);

  while !compr_reader.eof() {
    let mut buf = ~[0, ..256];
    let buf_len = buf.len();
    let read = compr_reader.read(buf, buf_len);
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
    Ok(*decoder.close())
  } else {
    Err(fmt!("decoder has not finished before end of input"))
  }
}

fn test_zlib(expected_data: &[u8], compr_reader: @io::Reader)
  -> Result<(), ~str> 
{
  match run_zlib(compr_reader) {
    Ok(result) =>
      if expected_data == result {
        Ok(())
      } else {
        Err(fmt!("decoder returned wrong result!!!"))
      },
    Err(err) => Err(err)
  }
}

fn bench_zlib(compr_reader: @io::Reader)
  -> Result<f64, ~str>
{
  let start_time = time::get_time();
  match run_zlib(compr_reader) {
    Ok(_result) => {
      let end_time = time::get_time();
      let delta_sec = end_time.sec - start_time.sec;
      let delta_nsec = end_time.nsec - start_time.nsec;
      Ok(delta_sec as f64 + delta_nsec as f64 * 1.0e-9)
    },
    Err(err) => Err(err),
  }
}

fn show(msg: &str) {
  io::stdout().write(msg.as_bytes());
  io::stdout().flush();
}
