require 'json'

filenames = Dir.glob('*.json')

filenames.each do |fn|

  f = File.open(fn)
  data = JSON.load(f)

  new_f_name = File.basename(f, '.json') + '.txt'

  File.open(new_f_name, 'w') do |ff|
    data.keys.each do |k|
      ff.write(k + "\n")
    end
  end

end