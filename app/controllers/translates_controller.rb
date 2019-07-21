require 'open-uri'

class TranslatesController < ApplicationController

  def show
  end

  def create
    @images = []

    doc = Nokogiri::HTML( open( params['url'] ) )

    doc.search('script').each do |script|
      next if script.children.empty?
      c = script.children

      washed_text = c.text.gsub( /[\n\t]/, '' )

      m = washed_text.match( /fleet:.({.*}),stock/ )
      next unless m

      data = JSON.parse( m[1] )

      File.open('tmp/fleet.txt','w') do |f|
        PP.pp(data,f)
      end

      data['ships'].each do |ship|
        ship_pics = []
        download_pic ship['image']
        ship_pics << ship['image']

        ship['upgrades'].each do |_, upgrade|
          next unless upgrade
          # p upgrade
          download_pic upgrade['image']
          ship_pics << upgrade['image']
        end

        @images << ship_pics
      end

      squadrons_pics = []
      data['squadrons'].each do |squadron|
        download_pic squadron['image']
        squadrons_pics << squadron['image']
      end

      @images << squadrons_pics
    end
  end

  private

  def download_pic( pic_name )
    FileUtils.mkpath( 'public/pics' )

    local_path = "public/pics/#{pic_name}"

    unless File.exist?( local_path )
      open( "https://armada.ryankingston.com/img/cards/#{pic_name}") do |image|
        File.open("public/pics/#{pic_name}", 'wb') do |file|
          file.write(image.read)
        end
      end
    end
  end

end
