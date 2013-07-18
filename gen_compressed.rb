require 'zlib' rescue LoadError
require 'stringio'
require 'timeout'

class ZlibInflate
  def self.vendor
    "zlib"
  end

  def self.format
    :zlib
  end

  def self.supported?
    const_defined? :Zlib
  end

  def initialize(level)
    @level = level
  end

  def name
    "zlib_zlib_#@level"
  end

  def compress(data)
    Zlib::Deflate.deflate(data, @level)
  end
end

class ZlibGzip
  def self.vendor
    "zlib"
  end

  def self.format
    :gzip
  end

  def self.supported?
    const_defined? :Zlib
  end

  def initialize(level)
    @level = level
  end

  def name
    "zlib_gzip_#@level"
  end

  def compress(data)
    sio = StringIO.new
    gz = Zlib::GzipWriter.new(sio)
    gz.write(data)
    gz.close()
    sio.string
  end
end

class SevenGzip
  PROGRAM = "7z"
  
  def self.vendor
    "7z"
  end

  def self.format
    :gzip
  end

  def self.supported?
    IO.popen [PROGRAM] { true } rescue Errno::ENOENT false
  end

  def initialize
  end

  def name
    "7z_gzip"
  end

  def compress(data)
    str = ""
    Timeout.timeout(10) do
      IO.popen [PROGRAM, "a", "dummy.gz", "-so", "-si"], "r+", :err => :close do |seven_z|
        seven_z.write(data)
        seven_z.flush()
        seven_z.close_write()

        until seven_z.eof?
          str << seven_z.read
        end
      end
    end
    str
  end
end

class Generator
  SUFFIXES = {
    :gzip => "gz",
    :zlib => "zlib",
  }

  def initialize(source_dir, target_dir, compressors)
    @source_dir = source_dir
    @target_dir = target_dir
    @compressors = compressors
  end

  def generate(sample)
    source_data = File.read(File.join(@source_dir, sample))
    @compressors.each do |compr|
      compr_dir = File.join(@target_dir, compr.name)
      compr_filename = "#{sample}.#{SUFFIXES[compr.class.format]}"
      compr_path = File.join(compr_dir, compr_filename)

      unless Dir.exists? compr_dir
        Dir.mkdir(compr_dir)
      end

      compr_data = compr.compress(source_data)
      File.write(compr_path, compr_data)
    end
  end
end

samples_dir = File.expand_path("../samples", __FILE__)
target_dir = File.expand_path("../compressed", __FILE__)
methods = {
  ZlibInflate => [[0], [6]],
  ZlibGzip => [[5], [9]],
  SevenGzip => [[]],
}

def show(msg)
  STDOUT.print(msg)
  STDOUT.flush
end

show "loading compressors:\n"
compressors = []
methods.each_pair do |klass, argss|
  show "  #{klass.vendor} for #{klass.format} ..."
  if klass.supported?
    show " ok\n"
    compressors.concat(argss.map { |args| klass.new(*args) })
  else
    show " not supported\n"
  end
end

gen = Generator.new(samples_dir, target_dir, compressors)
sample_names = Dir.entries(samples_dir).select { |s| s !~ /^[._]/ }.sort

unless Dir.exists? target_dir
  Dir.mkdir(target_dir)
end

show "compressing samples:\n"
sample_names.each do |sample|
  show "  #{sample} ..."
  gen.generate(sample)
  show " ok\n"
end

show "done\n"
