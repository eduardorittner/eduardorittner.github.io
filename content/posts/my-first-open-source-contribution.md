+++
title = 'My First Open Source Contribution'
date = 2023-09-21T14:47:56-03:00
draft = false
+++

Yesterday my first ever PR to an open source project was accepted and [commited](https://github.com/neovim/neovim/commit/b6ef938c663bc930a42bf1b15d7e97efcd4904b6), however, my contribution had (almost) nothing to do with code, but instead documentation, and I'm perfectly fine with that, here's why.

I'm relatively new to programming, in fact, my journey into coding started less than 2 years ago, when I got into university and started my undergrad in computer science and engineering. Since then I've had basically one coding discipline per semester (the first 2 years are kind of slow since I have to take a lot of engineering disciplines such as physics (1 to 4), a lot of maths and just a bit of chemistry).

The first semester's course was with python and focused on getting accustomed to the logic of programming: loops, variables, conditional statements, arrays, the list goes on. After that I had an algorithms and data structures course in C, which I thoroughly, enjoyed both the language and the assignments. My third semester was in Java where we learnt about OOP, which wasn't exactly my favorite, don't get me wrong, some of OOP's principles are good principles that I try to follow, but I wouldn't classify Java as a "fun" language to use. And now I'm currently learning assembly language (Risc-V) and the basics of computer architecture.

All that to say that I don't have a lot of experience with real world software, of course I've completed a ton of assignements, and they're mostly a joy to work on, but most of them are pretty basic and seem to be pretty far away from all the complexity that a real app (or even cli) has to handle in order to work well. So I've always had this interest in seeing how the things and apps I use everyday really trully work and how they choose to handle all the complexities and all the structures they rely on to do so.

Coupled with the desire to contribute to OSS that almost every developer and you get quite a lot of motivation. The only thing that was missing for some months was a project to which contribute, I wanted to work on something that I personally used and relied on, and was written in C. Most apps nowadays aren't written in C, and those that are are quite complex, like the linux kernel (so scary!) and whatnot.

I had my lucky break when I found out that Neovim, the editor I've been using for almost a year, was written in C. This, along with my natural interest of text editors in general led me to want to contribute to neovim. After having read the contributing guidelines, cloned the repo, compiled it and messed around for a few days with the source code, I went looking for issues in github to fix. I regularly checked up on them every day, but to no avail. After two weeks or so, I finally found a task I knew was I was capable of doing: updating the docs!.

It may sound quite silly but I was genuinely excited, it was just a case of adding the default values of other XDG\_ENV\_VARIABLES, such as XDG\_CACHE\_DIR to the documentation. To do that I just had to look into the actual code and find their definition, add that to the corresponding entry in starting.txt and issue a pull request.

I had some questions which I expressed on github (someone even complimented them!), and most were specifically regarding windows, which didn't seem to have XDG\_CONFIG\_DIRS and XDG\_DATA\_DIRS, and the people there were super nice and understanding. Having addressed them and a few formatting mistakes, my PR was accepted and merged!.

All of this to say that I'm not ashamed that my first contribution to neovim was on documentation. Contribution to open source involves a lot of things other than just code, and having a simple change that did not affect the inner workings of neovim allowed me to focus on learning the whole process of making a PR, all the git stuff you have to do, looking at someone else's source code, etc. And I'm happy I did it.

Now the next step of course is to contribute to the source code, and I will bring updates here whenever that happens.

