require 'open-uri'
require 'pp'
require 'nokogiri'
require 'json'
require 'fileutils'

class DataReader

	attr_reader :ships_images, :name, :total, :squadrons_pics, :squadron_ratio, :squadrons_total
  attr_reader :assault, :defense, :navigation

	def download( url )
		# puts 'About to download'

		download_data( url )

		@name = @data['name']
		@total = @data['points']['total']
		@squadrons_total = @data['points']['squadron']
		@squadron_ratio = ((@data['points']['squadron'].to_f * 100) / @total.to_f).round(1)

		process_ships
		process_squadrons

		@assault = @data['assault']['image']
    download_pic @assault
    @defense = @data['defense']['image']
    download_pic @defense
    @navigation = @data['navigation']['image']
    download_pic @navigation

		self
	end

	private

	def process_squadrons
		@squadrons_pics = []

		@data['squadrons'].each do |squadron|
			download_pic squadron['image']

			1.upto( squadron['quantity'].to_i ).each do
				@squadrons_pics << OpenStruct.new( { klass: :squadron, pic: squadron['image'] } )
			end
		end
	end

	# Find all ships and upgrades and link them to corresponding pictures
	def process_ships
		@ships_images = []

		@data['ships'].each do |ship|
			ship_pics = []
			download_pic ship['image']
			ship_pics << OpenStruct.new( { klass: :ship, pic: ship['image'] } )

			ship['upgrades'].each do |_, upgrade|
				next unless upgrade
				# p upgrade
				download_pic upgrade['image']
				ship_pics << OpenStruct.new( { klass: :upgrade, pic: upgrade['image'] } )
			end

			@ships_images << ship_pics
		end
	end

	# Download and parse data
	def download_data( url )
		@doc = Nokogiri::HTML( URI.open( url ) )

		# pp @doc

		@doc.search('script').each do |script|
			next if script.children.empty?
			c = script.children

			washed_text = c.text.gsub( /[\n\t]/, '' )

			m = washed_text.match( /fleet:.({.*}),stock/ )
			next unless m

			@data = JSON.parse( m[1] )

			# pp data

			File.open('/tmp/fleet.txt','w') do |f|
				PP.pp(@data,f)

				# pp @data
			end

      File.write('/tmp/fleet.json', JSON.pretty_generate(@data))
    end
	end

	# Download picture if not already downloaded
	def download_pic( pic_name )
		FileUtils.mkpath( 'pics' )

		local_path = "pics/#{pic_name}"

		unless File.exist?( local_path )
			URI.open( "https://armada.ryankingston.com/img/cards/#{pic_name}") do |image|
				File.open("pics/#{pic_name}", 'wb') do |file|
					file.write(image.read)
				end
			end
		end
	end

end