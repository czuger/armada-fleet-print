require 'pp'
require 'json'
require 'nokogiri'

doc = File.open('en/data.html') { |f| Nokogiri::HTML(f) }

result = {}

doc.search('.card').each do |card|
  name = card.children.first.children.to_s
  image = card.children[1]['style']
  points = card.children[2].children.to_s.to_i

  p(name, image, points)

  image.slice!('background-image: url("/img/cards/')
  image.slice!('");')

  result[name] = {name: name, image: image, points: points}
end

File.write('empire-squadrons.json', JSON.pretty_generate(result))
