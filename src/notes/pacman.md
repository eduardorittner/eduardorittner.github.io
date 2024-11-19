+++
title = 'Pacman'
date = 2024-08-06T17:52:43Z
draft = false
+++

# Removing packages

When removing packages, instead of using `pacman -R <package>` it's better to use `pacman -Rs <package>`, where the "-s" flag tells pacman to delete all depencies of \<package\> IF they were not installed explictly by the user and are not required by any other package. Not using this option leads to orphan packages, which are just a waste of space on your system.
To remove all orphaned packages, run `pacman -Qdtq | pacman -Rns`. Note: The second command may require authentication, and in that case just type in `sudo` right after the pipe: `pacman -Qdtq | sudo pacman -Rns`.
