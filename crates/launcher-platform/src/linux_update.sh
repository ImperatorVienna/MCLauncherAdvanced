#!/bin/sh
set -eu
pid=$1
destination=$2
staged=$3
backup=$4
ready=$5
: > "$ready"
count=0
while kill -0 "$pid" 2>/dev/null; do
    count=$((count + 1))
    [ "$count" -le 120 ] || exit 1
    sleep 1
done
# Work on the same filesystem; never truncate the running or backup executable.
if [ -f "$backup" ]; then rm -- "$backup"; fi
mv -- "$destination" "$backup"
if ! mv -- "$staged" "$destination"; then
    mv -- "$backup" "$destination"
    exit 1
fi
cd -- "$(dirname -- "$destination")"
# AppImage variables describe the old mounted image and must not leak to the new one.
unset APPIMAGE APPDIR OWD ARGV0
if ! "$destination"; then
    echo 'Updated launcher exited with an error; the previous executable is retained.' >&2
    exit 1
fi
