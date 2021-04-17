require 'rubygems'
require 'action_view'
require 'sinatra'
require 'sinatra/reloader' if development?
require 'haml'

require_relative 'libs/data_reader'

include ActionView::Helpers

get '/' do
  haml :show
end

post '/print' do
  # p params

  dr = DataReader.new.download( params['url'] )
  # dr = DataReader.new.download( 'https://armada.ryankingston.com/fleet/136865/' )

  @ships_images = dr.ships_images

  @name = dr.name
  @total = dr.total

  @squadron_ratio = dr.squadron_ratio
  @squadrons_pics = dr.squadrons_pics

  haml :print
end
