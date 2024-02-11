#!/usr/bin/sh

git init
git add *
git commit -m "initial commit"
git branch -M main
git remote add origin https://github.com/johnietre/logme.git
git push -u origin main
