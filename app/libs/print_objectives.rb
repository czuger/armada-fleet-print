def print_objectives(dr, pdf_doc, y_pos)
  # Print squadrons
  printed_squadrons = Set.new
  x_pos = 0

  x_factor = 183
  y_factor = 350

  objectives =[dr.assault, dr.defense, dr.navigation]

  objectives.each do |pic|
    pdf_doc.image "pics/#{pic}", width: 180, at: [0 + x_pos * x_factor, 700 - y_pos * y_factor]

    x_pos += 1
  end
end
