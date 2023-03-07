#! /bin/bash

VERSION=$(grep "version =" pyproject.toml | awk -F\" '{ print $2 }')
dpkg-buildpackage -b -uc -us
dpkg -b debian/aptupdatechecker
mv debian/aptupdatechecker.deb aptupdatechecker-$VERSION.deb