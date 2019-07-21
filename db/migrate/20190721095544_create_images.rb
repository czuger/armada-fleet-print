class CreateImages < ActiveRecord::Migration[5.2]
  def change
    create_table :images do |t|
      t.string :name, index: { unique: true }

      t.timestamps
    end
  end
end
