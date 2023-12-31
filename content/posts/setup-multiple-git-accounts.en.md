+++
title = 'TIL How To Setup Multiple Git Accounts'
date = 2023-09-11T20:15:41-03:00
draft = false
+++

I'm part of a junior company at my university where we design websites and web apps for small businesses. To keep everything neat and tidy, everyone gets an email account associated to the company (just like a company email minus the part where I get paid) to use all our tools such as figma for prototyping, google drive for file and document storage, and most importantly github for developing and maintaining projects. Configuring 2 accounts for github on the same machine is not some novel or complicated concept, and this post is just an amalgamation of what I've found online, to make it easier whenever I switch distros or have to rebuild my environment from scratch for any reason. Note that all of these steps are for linux based distros, and some of them might be different on Windows or MacOs.
So before we get to it, let's recap what I want this config to do:
1. Run all git commands without any authentication prompt
2. Automatically determine which github account to use
3. Allow for easy tweaks (such as adding new accounts)

For authentication we are going to be using SSH, since it's easy to set up and you only have to do it once per machine and never worry about it again. So for each account we have to generate a new ssh key, add it to the current agent and add it to the list of trusted keys in the github website. For generation we are going to be using the ssh-keygen command (which as far as I'm aware is available by default in the vast majority of operating systems, just be careful of syntax differences) as follows:
```
ssh-keygen -t rsa -C "your-email@domain.com" -f "your-github-username"
```
Ssh-keygen generates a new ssh key, where the t flag specifies which key type to build, the -C flag adds a comment to the key (just as a way to know which email it's linked to) and the -f flag specifies the file name to save the key in the ~/.ssh directory. Upon entering the above command, it will prompt you for a passphrase, you can either chose to enter one or just leave it empty, my choice was to leave it empty just because it seemed like less of a hassle. For those of you unaware, rsa encryption involves 2 sets of keys, a private and a public key (denoted by the ".pub" terminator), the private key is only stored locally and the public key is given to whomever we want to ssh to, in this case, github. So having generated our keys, we must add our new private key identity to the ssh agent
```
ssh-add -K ~/.ssh/key-name
```
To be honest I'm not really sure what the -K flag does, I've tried looking into it and it has something to do with FIDO authentication but I'm not sure if it's necessary, just an extra precaution, or what the deal is. Anyways, having generated our keys and added them to the ssh-agent, we must now tell github to accept it as a valid ssh key. For that you simply go into Settings, SSG and GPG keys and click on new SSH key and paste your public ssh key (that is, the file "key-name.pub") there, and voila.
You must follow these steps for each and every account you wish to configure, generating a new key for each one and adding it to the ssh-agent and corresponding github account valid keys. After all that, we will now tell git how to know which identity to use, and for that we're going to modify (or create) the ~/.gitconfig file. As far as I'm aware there are several ways to do it, but the one that worked best for me was to have one specific directory for each github account, that way git knows which identity to use based on where the repo I'm currently working on is located. Another particular decision was to have a separate .gitconfig file in each of these parent directories, so that I can change configs for each account separately.
My .gitconfig file is pretty barebones, and looks something like this:
```
[core]
editor = nvim

[includeIf "gitdir:~/personal/"]
path = ~/personal/.gitconfig

[includeIf "gitdir:~/work/"]
path = ~/work/.gitconfig
```
Up top are all the global settings you want to use, in my case that's just setting the default editor, and after that comes the local configurations for each account. IncludeIf does just that, include the configuration that is on path if the current git directory is inside the specified folder (note that a git repo doesn't have to be directly on the specified directory to work, it can be nested inside however many folders). Having done that we must now create the specified config files for each account:
```
[user]
email = email-name@domain.com
name = name

[github]
user = "user-name"

[core]
sshCommand = "ssh -i ~/.ssh/key-name"
```
All this does is specify the current user, name and email, and which ssh key to use for the ssh authentication, and that's it. If you'd like to have settings specific to one account you'd place them in this file, however that's not my case. And that's it! you're all ready to go, just note that whenever you're cloning a repo now you should use the ssh option provided in the code button, rather than copying the URL directly, for example (it sometimes works but just for consistency, I've gotten used to just using the ssh option).
