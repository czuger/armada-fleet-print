require 'open-uri'
require 'ostruct'

class TranslatesController < ApplicationController

  def show
  end

  def create
    dr = DataReader.new.download( params['url'] )

    @ships_images = dr.ships_images

		@name = dr.name
		@total = dr.total

		@squadron_ratio = dr.squadron_ratio
		@squadrons_pics = dr.squadrons_pics
  end

end
