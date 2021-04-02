require 'prawn'
require_relative 'data_reader'


class PdfCreator

  def initialize
    # dr = DataReader.new.download( params['url'] )

  end

  def print_squadrons(dr)

  end

  def create_pdf
    dr = DataReader.new.download( 'https://armada.ryankingston.com/fleet/136912/' )

    Prawn::Document.generate('result.pdf') do

      dr.ships_images.each_with_index do |images_list, ship_index|

        p images_list

        x_pos = 210
        ship_cursor_pos = cursor

        images_list.each do |img_data|

          # p img_data

          if img_data.klass == :upgrade
            image "pics/#{img_data.pic}", width: 100, height: 150, at: [x_pos, ship_cursor_pos]
            x_pos += 110
          else
            ship_cursor_pos = cursor
            image "pics/#{img_data.pic}", width: 200
            move_down 10
          end
        end

        # p ship_index
        # p (ship_index % 2) == 0

        if ship_index % 2 == 1
          start_new_page
        end
      end

      # Print squadrons
      printed_squadrons = Set.new

      start_new_page
      # move_up 600

      x_pos = 0
      y_pos = 0

      x_factor = 210
      y_factor = 350

      dr.squadrons_pics.each_with_index do |squadron, ship_index|

        p squadron
        # p img_data

        # p x_pos * x_factor, y_pos * y_factor

        if !printed_squadrons.include?(squadron.pic)
          image "pics/#{squadron.pic}", width: 200, at: [50 + x_pos * x_factor, 700 - y_pos * y_factor]
          printed_squadrons << squadron.pic
          x_pos += 1

          if x_pos >= 2
            y_pos += 1
            x_pos = 0

            if y_pos >= 2
              y_pos = 0
              start_new_page
            end
          end
        end
      end
    end
  end
end

PdfCreator.new.create_pdf
