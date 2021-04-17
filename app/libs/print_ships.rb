def print_ships(dr, pdf_doc)
  pages = 1
  dr.ships_images.each_with_index do |images_list, ship_index|

    p images_list

    x_pos = 210
    ship_cursor_pos = pdf_doc.cursor

    images_list.each do |img_data|

      # p img_data

      if img_data.klass == :upgrade
        pdf_doc.image "pics/#{img_data.pic}", width: 100, height: 150, at: [x_pos, ship_cursor_pos]
        x_pos += 110
      else
        ship_cursor_pos = pdf_doc.cursor
        pdf_doc.image "pics/#{img_data.pic}", width: 200
        pdf_doc.move_down 10
      end
    end

    # p ship_index
    # p (ship_index % 2) == 0

    if ship_index % 2 == 1
      pdf_doc.start_new_page
      pages += 1
    end


  end

  pages
end