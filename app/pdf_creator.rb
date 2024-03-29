require 'prawn'
require_relative 'libs/data_reader'
require_relative 'libs/print_squadrons'
require_relative 'libs/footer'
require_relative 'libs/print_ships'
require_relative 'libs/print_objectives'

# TODO : Count lines, to avoid half blank page.

class PdfCreator

  def initialize
    # dr = DataReader.new.download( params['url'] )

  end

  def create_pdf(url)
    dr = DataReader.new.download( url )

    Prawn::Document.generate('/tmp/fleet.pdf') do |pdf_doc|
      ships_count = print_ships(dr, pdf_doc)

      y_pos = 0
      if(ships_count % 2 == 0)
        pdf_doc.start_new_page
      else
        y_pos = 1
      end

      # move_up 600

      y_pos = print_squadrons(dr, pdf_doc, y_pos)
      print(y_pos)

      if y_pos >= 2
        y_pos = 0
        pdf_doc.start_new_page
      end
      print_objectives(dr, pdf_doc, y_pos)

      footer(dr, pdf_doc)
    end
  end
end

if __FILE__ == $0
  # PdfCreator.new.create_pdf 'https://armada.ryankingston.com/fleet/207536/'
  PdfCreator.new.create_pdf 'https://armada.ryankingston.com/fleet/158694/'
end