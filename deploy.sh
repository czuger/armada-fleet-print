#!/usr/bin/env bash

scp -r app cloud_ovh:~/ruby/armada-fleet-print/
scp .ruby-version cloud_ovh:~/ruby/armada-fleet-print/
scp Gemfile cloud_ovh:~/ruby/armada-fleet-print/
scp Gemfile.lock cloud_ovh:~/ruby/armada-fleet-print/
scp prod.sh cloud_ovh:~/ruby/armada-fleet-print/
