#!/bin/bash
set -e

if [[ "$#" -ne 1 ]]; then 
	export GENERIC_LAUNCHER_INSTALL_DIR="$(dirname "$0")"
else
	export GENERIC_LAUNCHER_INSTALL_DIR="$1"
fi

if [[ ! -d "$GENERIC_LAUNCHER_INSTALL_DIR" ]]; then 
	echo "Error setting installing dir"
fi

echo "installing to $GENERIC_LAUNCHER_INSTALL_DIR"
project_dir=$(dirname "$0")
cd "$project_dir"
cargo build
set -x
if ! install -v ./target/debug/generic_launcher "$GENERIC_LAUNCHER_INSTALL_DIR"; then 
	echo $?
fi
if [[ $(realpath "$project_dir") !=  $(realpath "$GENERIC_LAUNCHER_INSTALL_DIR") ]]; then
	cp -r "$project_dir/assets" "$GENERIC_LAUNCHER_INSTALL_DIR"
	cp "$project_dir/launcher.css" "$GENERIC_LAUNCHER_INSTALL_DIR"
fi
"$GENERIC_LAUNCHER_INSTALL_DIR/generic_launcher"