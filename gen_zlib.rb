require 'zlib'

def show(msg)
  STDOUT.print msg
  STDOUT.flush
end

samples_dir = File.expand_path("../samples", __FILE__)
levels = [0, 6, 9]

show "reading samples "
sample_names = Dir.entries(samples_dir).select { |s| s[0..0] != "." }

samples = {}
sample_names.each do |sample_name|
  sample = File.read(File.join(samples_dir, sample_name))
  samples[sample_name] = sample
  show "."
end
show " ok\n"

levels.each do |level|
  show "level #{level} "
  level_dir = File.expand_path("../zlib/zlib#{level}", __FILE__)
  Dir.mkdir(level_dir) unless Dir.exists? level_dir

  samples.each do |name, data|
    compressed = Zlib::Deflate.deflate(data, level)
    File.write(File.join(level_dir, "#{name}.zlib"), compressed)
    show "."
  end
  show " ok\n"
end
