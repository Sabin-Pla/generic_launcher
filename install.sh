#!/bin/bash

if [[ "$#" -ne 1 ]]; then 
	export GENERIC_LAUNCHER_INSTALL_DIR="$(dirname "$0")"
else
	export GENERIC_LAUNCHER_INSTALL_DIR="$1"
fi

if [[ ! -d "$GENERIC_LAUNCHER_INSTALL_DIR" ]]; then 
	echo "Error setting installing dir"
fi

echo "installing to $GENERIC_LAUNCHER_INSTALL_DIR"
