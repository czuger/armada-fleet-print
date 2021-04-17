def print_squadrons(dr, pdf_doc)
  # Print squadrons
  printed_squadrons = Set.new
  x_pos = 0
  y_pos = 0

  x_factor = 210
  y_factor = 350

  dr.squadrons_pics.each_with_index do |squadron, ship_index|

    p squadron
    # p img_data

    # p x_pos * x_factor, y_pos * y_factor

    if !printed_squadrons.include?(squadron.pic)
      pdf_doc.image "pics/#{squadron.pic}", width: 200, at: [50 + x_pos * x_factor, 700 - y_pos * y_factor]
      printed_squadrons << squadron.pic
      x_pos += 1

      if x_pos >= 2
        y_pos += 1
        x_pos = 0

        if y_pos >= 2
          y_pos = 0
          pdf_doc.start_new_page
        end
      end
    end
  end
end
