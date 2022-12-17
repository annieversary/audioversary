#!/bin/bash

set -e

if [[ -z $1 ]]; then
    echo "Builds, bundles and copies to /Library/Audio/Plug-Ins/VST3"
    echo "Example:"
    echo -e "\t$0 packagename"
else
    cargo xtask bundle "$1"
    echo "Built"
    sudo cp -r "target/bundled/$1.vst3" /Library/Audio/Plug-Ins/VST3
    echo "Copied"
fi
