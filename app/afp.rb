# myapp.rb
require 'sinatra'
require_relative 'pdf_creator'

settings = File.read('config/settings.json')
settings = JSON.parse(settings)

set :port, settings['port']

get '/' do
  haml :show
end

post '/print' do
  PdfCreator.new.create_pdf params['url']
  send_file '/tmp/fleet.pdf'
end