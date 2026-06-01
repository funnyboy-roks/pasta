#!/bin/bash

set -xe

# this is a kind of nasty hack to make the api change at runtime, rather than compile-time
cp -r /pasta-ui /pasta-ui-fixed
cd /pasta-ui-fixed
find -type f -exec sed -i "s|{{PASTA_API}}|$API|g" '{}' \;

exec /pasta
