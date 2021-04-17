require 'prawn'
require_relative 'libs/data_reader'
require_relative 'libs/print_squadrons'
require_relative 'libs/footer'
require_relative 'libs/print_ships'

# TODO : Count lines, to avoid half blank page.
# White circle + black number for squadrons.

class PdfCreator

  def initialize
    # dr = DataReader.new.download( params['url'] )

  end

  def create_pdf(url)
    dr = DataReader.new.download( url )

    Prawn::Document.generate('/tmp/fleet.pdf') do |pdf_doc|
      pages_count = 0

      pages_count += print_ships(dr, pdf_doc)

      # p(pages_count)
      # p(pages_count % 2 == 0)


      pdf_doc.start_new_page

      # move_up 600

      print_squadrons(dr, pdf_doc)

      footer(dr, pdf_doc)
    end
  end
end
