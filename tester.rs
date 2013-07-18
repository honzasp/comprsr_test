extern mod extra;
extern mod comprsr_zlib
  (vers = "0.0.1", author = "github.com/honzasp");

use std::io;
use std::os;
use std::hashmap;
use std::iterator;
use std::from_str;
use extra::time;
use _show::*;

mod zlib { pub use comprsr_zlib::zlib::*; }

fn main() {
  let test_root = match os::self_exe_path() {
      Some(self_path) => self_path,
      None => fail!(~"os::self_exe_path() returned None"),
    };

  let default_chunk_size = 1024;
  let chunk_size: uint = match os::getenv("CHUNK_SIZE") {
    Some(envvar) => {
      let opt_uint: Option<uint> = from_str::FromStr::from_str(envvar);
      match opt_uint {
        Some(chunk_size) if chunk_size > 0 => chunk_size,
        _ => default_chunk_size,
      }
    },
    None => default_chunk_size,
  };

  let samples_dir = &test_root.push("samples");
  let compr_dir = &test_root.push("compressed");

  show_info(fmt!("using chunks of %u bytes\n", chunk_size));
  show_info("reading samples ");
  let list_dir = os::list_dir_path(samples_dir);
  let samples: &hashmap::HashMap<~str, ~[u8]> =
    &iterator::FromIterator::from_iterator(&mut list_dir.iter().filter_map(
      |sample_path| 
    {
      let name = sample_path.filename().unwrap();
      if name.char_at(0) != '.' && name.char_at(0) != '_' {
        let data = match io::read_whole_file(*sample_path) {
            Ok(data) => data,
            Err(err) => fail!(fmt!("reading %? failed: %?", sample_path, err)),
          };
        show_file(".");
        Some((name, data))
      } else {
        None
      }
    }));
  show_ok(" ok\n");

  let subpaths = os::list_dir_path(compr_dir);
  let mut total_time: f64 = 0.0;
  let mut total_errors: uint = 0;

  for subpaths.iter().advance |subdir| {
    if os::path_is_dir(*subdir) {
      let (time, errors) = process_subdir(*subdir, samples, chunk_size);
      total_time += time;
      total_errors += errors;
    }
  };

  show_info(   fmt!(" = total: ======\n"));
  show_ok(     fmt!("  %-10s s\n", total_time.to_str()));
  if total_errors > 0 {
    show_error(fmt!("  %5s errors\n", total_errors.to_str()));
  }
}

fn process_subdir(
  dir: &Path,
  samples: &hashmap::HashMap<~str, ~[u8]>,
  chunk_size: uint)
-> (f64, uint)
{
  show_info("# ");
  show_dir(fmt!("%s\n", dir.filename().unwrap()));

  let mut total_time: f64 = 0.0;
  let mut errors: uint = 0;

  let subpaths = os::list_dir_path(dir);
  for subpaths.iter().advance |compr_path| {
    let sample_name = compr_path.filestem().unwrap();
    let sample_data = samples.find(&sample_name).unwrap();

    show_file(fmt!("  %-40s ", sample_name));
    let compr_data = match io::read_whole_file(*compr_path) {
      Ok(data) => data,
      Err(err) => fail!("Could not read %s: %s", compr_path.to_str(), err.to_str()),
    };

    let format = suffix_to_format(compr_path.filetype());
    match process_file(format, *sample_data, compr_data, chunk_size) {
      Ok(time) => {
        show_ok(fmt!("%-10s s\n", time.to_str()));
        total_time += time;
      },
      Err(err) => {
        show_error(fmt!("err!: %s\n", err.to_str()));
        errors += 1;
      },
    };
  }

  show_info(   fmt!("  %-40s--------------\n", ""));
  show_ok(     fmt!("  %-40s %-10s s\n", "", total_time.to_str()));
  if errors > 0 {
    show_error(fmt!("  %-40s %5s errors\n", "", errors.to_str()));
  }
  show_ok("\n");

  (total_time, errors)
}

enum Format {
  ZlibFormat,
  GzipFormat,
}

fn suffix_to_format(suffix: Option<~str>) -> Format {
  match suffix {
    Some(~".zlib") => ZlibFormat,
    Some(~".gz") => GzipFormat,
    Some(other) => fail!("Unknown filetype %?", other),
    None => fail!("Missing suffix"),
  }
}

fn decompress(format: Format, compr_data: &[u8], chunk_size: uint)
  -> Result<~[u8], ~str> 
{
  match format {
    ZlibFormat => decompress_zlib(compr_data, chunk_size),
    GzipFormat => decompress_gzip(compr_data, chunk_size),
  }
}

fn process_file(
  format: Format,
  expected_data: &[u8],
  compr_data: &[u8],
  chunk_size: uint)
  -> Result<f64, ~str>
{
  let start_time = time::get_time();
  match decompress(format, compr_data, chunk_size) {
    Ok(result) => {
      let end_time = time::get_time();
      let delta_sec = end_time.sec - start_time.sec;
      let delta_nsec = end_time.nsec - start_time.nsec;
      if result.as_slice() == expected_data {
        Ok(delta_sec as f64 + delta_nsec as f64 * 1.0e-9)
      } else {
        Err(fmt!("decoder returned wrong result!!!"))
      }
    },
    Err(err) => Err(err),
  }
}

fn decompress_zlib(compr_data: &[u8], chunk_size: uint)
  -> Result<~[u8], ~str> 
{
  let mut decoder = zlib::decoder::Decoder::new();
  let mut output = ~[];

  let mut iter = compr_data.chunk_iter(chunk_size);
  loop {
    let chunk = match iter.next() {
      Some(chunk) => chunk,
      None => break,
    };

    let (result, new_output) = decoder.input(chunk, output);
    output = new_output;
    match result {
      Left(new_decoder) => {
        decoder = new_decoder;
      },
      Right((outcome, rest)) => {
        match outcome {
          Ok(()) => {
            if rest.is_empty() {
              break
            } else {
              return Err(fmt!("decoder finished before end of input \
                (%u bytes left in input buffer, %u bytes in output)",
                rest.len(), output.len() ))
            }
          },
          Err(err) => {
            return Err(fmt!("decoder returned error: %s (%u bytes in output)",
              err.to_str(), output.len() ))
          }
        }
      },
    }
  }

  if iter.next().is_some() {
    Err(fmt!("decoder has not finished before end of input \
             (%u bytes in output)", output.len()))
  } else {
    Ok(output)
  }
}

fn decompress_gzip(_compr_data: &[u8], _chunk_size: uint)
-> Result<~[u8], ~str> 
{
  Err(~"gzip not implemented yet")
}

#[cfg(use_colors)]
mod _show {
  use extra::term;
  use std::io;

  pub fn show_ok(msg: &str)    { show_color(msg, term::color::GREEN); }
  pub fn show_error(msg: &str) { show_color(msg, term::color::RED); }
  pub fn show_info(msg: &str)  { ::show_plain(msg); }
  pub fn show_file(msg: &str)  { ::show_plain(msg); }
  pub fn show_dir(msg: &str)   { show_color(msg, term::color::BRIGHT_CYAN); }

  fn show_color(msg: &str, color: term::color::Color) {
    let out = io::stdout();
    match term::Terminal::new(out) {
      Ok(term) => {
        term.fg(color);
        out.write_str(msg);
        term.reset();
      },
      Err(_) => {
        ::show_plain(msg);
      },
    }
  }
}

#[cfg(not(use_colors))]
mod _show {
  pub fn show_ok(msg: &str)    { ::show_plain(msg); }
  pub fn show_error(msg: &str) { ::show_plain(msg); }
  pub fn show_info(msg: &str)  { ::show_plain(msg); }
  pub fn show_file(msg: &str)  { ::show_plain(msg); }
  pub fn show_dir(msg: &str)   { ::show_plain(msg); }
}

pub fn show_plain(msg: &str) {
  let out = io::stdout();
  out.write_str(msg);
  out.flush();
}
