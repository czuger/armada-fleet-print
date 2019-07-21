require 'test_helper'

class TranslateControllerTest < ActionDispatch::IntegrationTest

  test 'download test url' do
    post translates_url, params: { url: 'https://armada.ryankingston.com/fleet/53386/' }
  end

end
