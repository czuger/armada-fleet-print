require 'json'

ships_reb = File.open('rebels-ships.json')
ships_reb = JSON.load(ships_reb)

squad_reb = File.open('rebels-squadrons.json')
squad_reb = JSON.load(squad_reb)

reb = ships_reb.merge(squad_reb)

ships_emp = File.open('empire-ships.json')
ships_emp = JSON.load(ships_emp)

squad_emp = File.open('empire-squadrons.json')
squad_emp = JSON.load(squad_emp)

emp = ships_emp.merge(squad_emp)

result = {'rebels': reb, 'empire': emp}

File.open('units.json', 'w'){ |f| f.write(JSON.pretty_generate(result))}


