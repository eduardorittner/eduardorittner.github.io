+++
title = 'Telling my desktop launcher what terminal to use'
date = 2024-08-05T15:55:18Z
draft = false
+++

I use hyprland and tofi to run applications, and after installing [yazi](https://github.com/sxyazi/yazi) and trying to run it, tofi kept opening yazi inside gnome-terminal, and apparently this is due to desktop launchers having a hardcoded list of terminals [^1], and the first instance they find is the one launched. To circumvent this you can symlink your prefered terminal to a top-priority terminal, for example xterm. So if I want tofi to use kitty, I run the command `ln -s /usr/bin/kitty /usr/bin/xterm` and that's it. Just be careful to not symlink over a terminal you actually have installed.

[^1]: https://github.com/philj56/tofi/issues/46
