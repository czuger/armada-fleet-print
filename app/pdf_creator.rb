require_relative 'data_reader'

class PdfCreator

  def initialize
    # dr = DataReader.new.download( params['url'] )
    dr = DataReader.new.download( 'https://armada.ryankingston.com/fleet/136865/' )

    @ships_images = dr.ships_images

    @name = dr.name
    @total = dr.total

    @squadron_ratio = dr.squadron_ratio
    @squadrons_pics = dr.squadrons_pics
  end

end

PdfCreator.new