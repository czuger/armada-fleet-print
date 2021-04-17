def print_squadrons(dr, pdf_doc)
  # Print squadrons
  printed_squadrons = Set.new
  x_pos = 0
  y_pos = 0

  x_factor = 210
  y_factor = 350

  pdf_doc.font_size 24

  squadrons = {}
  dr.squadrons_pics.each_with_index do |squadron|
    squadrons[squadron] ||= 0
    squadrons[squadron] += 1
  end

  squadrons.each do |squadron_pic, count|
    pdf_doc.image "pics/#{squadron_pic.pic}", width: 200, at: [50 + x_pos * x_factor, 700 - y_pos * y_factor]

    pdf_doc.fill_color 'FFFFFF'
    pdf_doc.fill_ellipse [(50 + x_pos * x_factor)+180, (700 - y_pos * y_factor)-22], 15

    pdf_doc.fill_color '000000'
    pdf_doc.draw_text count, at: [(50 + x_pos * x_factor)+174, (700 - y_pos * y_factor)-30]

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
