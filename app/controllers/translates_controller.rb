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
        p ship['image']

        ship['upgrades'].each do |_, upgrade|
          next unless upgrade
          # p upgrade
          p upgrade['image']
        end
      end

      data['squadrons'].each do |squadron|
        p squadron['image']
      end

    end
  end

end
