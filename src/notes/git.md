+++
title = 'Notes on Git'
date = 2024-08-04T03:15:38Z
draft = false
+++

## 1. Merging unrelated repos

You can merge two entirely distinct and independent repos, which can be very useful. Saw this on one of Jon Gjenset's [streams](https://www.youtube.com/watch?v=xUH-4y92jPg&t=5104s&ab_channel=JonGjengset) where he basically has a repo with some general github actions for CI in rust crates, and whenever he creates a new repo for a crate, he adds the CI repo, merges it and gets all the github actions configured right away. The benefit of this strategy is that if somedat he updates the CI repo, adds some new action or changes an existing one, he can update any repo that uses them by just pulling and merging the changes.

### How to:

1. Add the repo as a remote with `git add remote <branch-name> <second-repo>`
2. Run `git fetch <branch-name>` to pull all changes
3. Run `git merge --allow-unrelated <branch-name>/main` to merge

And you're good to go. If at any point changes are made to the second repo, you just redo steps 2 and 3 again.

### Use cases:

The most useful use case for this is also the one on Jon Gjenset's stream, which is basically any generic configuration that you may want to reuse for more than one git repo. In his case, they were github actions for rust crates, but you could imagine for example a common .clang-format file that you want to use for all your c projects, or some common c headers you use all the time.
