#!/usr/bin/env bash

cd app
nohup bundle exec ruby afp.rb -o 0.0.0.0 &
