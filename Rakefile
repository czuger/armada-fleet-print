require 'standalone_migrations'
require 'rake/testtask'

StandaloneMigrations::Tasks.load_tasks

Rake::TestTask.new do |t|
  t.pattern = 'tests/test_*.rb'
end
