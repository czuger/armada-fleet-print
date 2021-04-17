def footer(dr, pdf_doc)
  pdf_doc.font_size 12

  page_string = 'page <page> of <total>'
  page_options = {
      at: [pdf_doc.bounds.right - 150, 0],
      width: 150,
      align: :right,
      start_count_at: 1,
  }
  pdf_doc.number_pages page_string, page_options

  name_string = "#{dr.name} (#{dr.squadrons_total}/#{dr.total})"
  name_options = {
      at: [pdf_doc.bounds.left, 0],
      align: :left,
  }
  pdf_doc.number_pages name_string, name_options
end