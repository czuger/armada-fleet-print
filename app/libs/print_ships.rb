def print_ships(dr, pdf_doc)
  pages = 1
  ships = 0

  dr.ships_images.each_with_index do |images_list, ship_index|

    # p images_list
    if ships >= 2
      pdf_doc.start_new_page
      pages += 1
      ships = 0
    end

    x_pos = 210
    ship_cursor_pos = pdf_doc.cursor

    images_list.each do |img_data|

      # p img_data

      if img_data.klass == :upgrade
        pdf_doc.image "pics/#{img_data.pic}", width: 100, height: 150, at: [x_pos, ship_cursor_pos]
        x_pos += 110

        if x_pos >= 210 + 110*3
          ship_cursor_pos -= 160
          x_pos = 210
        end

      else
        ship_cursor_pos = pdf_doc.cursor
        pdf_doc.image "pics/#{img_data.pic}", width: 200
        pdf_doc.move_down 10
      end
    end

    ships += 1
  end

  pages
end