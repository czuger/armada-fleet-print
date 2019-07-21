class TranslatesController < ApplicationController

  def show
  end

  def create
    doc = Nokogiri::HTML( open( params['url'] ) )

    # pp doc

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
        download_pic ship['image']

        ship['upgrades'].each do |_, upgrade|
          next unless upgrade
          # p upgrade
          download_pic upgrade['image']
        end
      end

      data['squadrons'].each do |squadron|
        download_pic squadron['image']
      end

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
