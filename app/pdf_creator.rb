require 'prawn'
require_relative 'data_reader'


class PdfCreator

  def initialize
    # dr = DataReader.new.download( params['url'] )

  end

  def create_pdf
    dr = DataReader.new.download( 'https://armada.ryankingston.com/fleet/134718/' )

    Prawn::Document.generate('result.pdf') do

      dr.ships_images.each_with_index do |images_list, i|

        x_pos = 300
        ship_cursor_pos = cursor

        images_list.each do |img_data|

          if img_data.klass == :upgrade
            image "pics/#{img_data.pic}", scale: 0.5, at: [x_pos, ship_cursor_pos]
            x_pos += 50
          else
            ship_cursor_pos = cursor
            image "pics/#{img_data.pic}", scale: 0.5
            move_down 10
          end

        end
      end
    end
  end

end

PdfCreator.new.create_pdf
