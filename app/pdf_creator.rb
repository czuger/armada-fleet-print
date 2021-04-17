require 'prawn'
require_relative 'libs/data_reader'
require_relative 'libs/print_squadrons'
require_relative 'libs/footer'
require_relative 'libs/print_ships'

# TODO : use explicit blocs for prawn to have functions.
# Count lines, to avoid half blank page.
# White circle + black number for squadrons.

class PdfCreator

  def initialize
    # dr = DataReader.new.download( params['url'] )

  end

  def create_pdf
    dr = DataReader.new.download( 'https://armada.ryankingston.com/fleet/138627/' )

    Prawn::Document.generate('result.pdf') do |pdf_doc|
      pages_count = 0

      pages_count += print_ships(dr, pdf_doc)

      pdf_doc.start_new_page
      # move_up 600

      print_squadrons(dr, pdf_doc)

      footer(dr, pdf_doc)
    end
  end
end

PdfCreator.new.create_pdf
