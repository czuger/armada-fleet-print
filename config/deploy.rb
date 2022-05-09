# config valid for current version and patch releases of Capistrano
lock '~> 3.17.0'

set :application, 'armada_fleet_print'
set :repo_url, 'git@github.com:czuger/armada-fleet-print.git'

set :rbenv_ruby, File.read('.ruby-version').strip
set :rbenv_prefix, '/usr/bin/rbenv exec'
set :rbenv_type, :user # or :system, depends on your rbenv setup
set :rbenv_ruby, '3.0.2'

set :deploy_to, '/home/webapp/ruby/armada_fleet_print'

append :linked_dirs, 'log', 'tmp/pids', 'tmp/cache', 'tmp/sockets', 'vendor/bundle', 'pics'
append :linked_files, 'db/config.yml', 'config/secrets.yml', 'config/settings.json'

task :restart_bot do
  on roles :all do
    supervisor_command_name = fetch(:supervisor_command_name)

    execute "supervisorctl restart #{supervisor_command_name}"
  end
end

after 'deploy:finished', :restart_bot

# Default branch is :master
# ask :branch, `git rev-parse --abbrev-ref HEAD`.chomp

# Default deploy_to directory is /var/www/my_app_name
# set :deploy_to, "/var/www/my_app_name"

# Default value for :format is :airbrussh.
# set :format, :airbrussh

# You can configure the Airbrussh format using :format_options.
# These are the defaults.
# set :format_options, command_output: true, log_file: "log/capistrano.log", color: :auto, truncate: :auto

# Default value for :pty is false
# set :pty, true

# Default value for :linked_files is []
# append :linked_files, "config/config.yml"

# Default value for linked_dirs is []
# append :linked_dirs, "log", "tmp/pids", "tmp/cache", "tmp/sockets", "public/system"

# Default value for default_env is {}
# set :default_env, { path: "/opt/ruby/bin:$PATH" }

# Default value for local_user is ENV['USER']
# set :local_user, -> { `git config user.name`.chomp }

# Default value for keep_releases is 5
# set :keep_releases, 5

# Uncomment the following to require manually verifying the host key before first deploy.
# set :ssh_options, verify_host_key: :secure
