require 'json'

filenames = Dir.glob('detail/*.txt')

result = {}
filenames.each do |fn|

  f = File.open(fn)
  lines = f.readlines

  side, typ = File.basename(fn, '.txt').split('_')

  result[side] ||= {}
  result[side][typ] ||= lines.map{ |e| e.strip() }
end

File.open('../final_data/names.json', 'w'){ |f| f.write(JSON.pretty_generate(result))}