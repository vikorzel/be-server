#!/bin/sh

# Install dependencies
sudo apt-get update
sudo apt-get install -fy cmake gcc c++ libssl-dev

# Build release
cd /app
cargo build --release

VERSION=`cat version.txt`
FOLDER=/app/tmp/be-server_${VERSION}_amd64
mkdir -p $FOLDER/usr/local/bin
mkdir -p $FOLDER/DEBIAN

cp /app/target/release/be-server $FOLDER/usr/local/bin/

rm $FOLDER/DEBIAN/control
touch $FOLDER/DEBIAN/control
echo "Package: be-server" >> $FOLDER/DEBIAN/control
echo "Version: $VERSION" >> $FOLDER/DEBIAN/control
echo "Architecture: amd64" >> $FOLDER/DEBIAN/control
echo "Maintainer: vikorzel hard.slot@gmail.com" >> $FOLDER/DEBIAN/control
echo "Description: BE Server for fridge contol" >> $FOLDER/DEBIAN/control

dpkg-deb --build --root-owner-group $FOLDER
echo "Done"

rm -rf $FOLDER