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
install -v ./target/debug/generic_launcher "$GENERIC_LAUNCHER_INSTALL_DIR"
"$GENERIC_LAUNCHER_INSTALL_DIR/generic_launcher"