#!/usr/bin/env bash

cd app
bundle exec rerun --pattern '**/*.rb' 'ruby afp.rb'