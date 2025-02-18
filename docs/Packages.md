# Packages in SVSM
  This document explains the idea behind 'package files' and how they are intended to work.

## Purpose
  Package files exist mostly to cover potential issues that may arise from Void's packaging system. While it works well,
there are certain issues that may arise from us trying to use it to define all packages. This is because of a few issues:

1. Restricted packages aren't listed if you do an `xbps-query` -Rs without them installed. So we can not know if a
   package (such as discord) exists or not unless we have the void-packages repository already pulled and discord built
   from it, in which case, we know it is from that repository.

2. Packages may or may not include information on what files are used to configure them. So we need to be able to define
   that package uses a file as its config file, without needing to do something hacky like inspect its template on
   void-packages.

3. Some packages, like bash, use multiple configuration files, or use variously named files. Bash, as an example, uses
   `.bash_profile` and `.bashrc` as its configuration files. However, the package doesn't declare that and as such SVSM
   would be unable to properly handle configuring those packages without some hacky logic or less-than-desirable
   solutions (like defining a configuration key that just takes a list of files to use as its config format or
   something similar.) As we want to keep the relatively easy-to-use and nice looking format we have currently.

4. Because packages are treated by the main file as a function, we need a way to create that function, but SVSL does not
   provide function definition or declaration semantics. As such, we need some way to generate or otherwise manage these
   cases, as we could make them built-in but that would quickly bloat SVSM and make updates required if a package
   changes or a new package needing special treatment is added.


As such, a way to define packages using the specific features of SVSM to allow us to change as little as possible while
getting things working was selected.

## Package File
   A package file is a simple SVSL file located in a specific directory, usually with `.pvsm` as the extension. It defines
the symbol for the package, along with the actual name to use with xbps. The symbol is defined as a map with keys
set for non-standard configuration options. These keys are then used to generate the way that SVSM will handle the calls
to the package functions at runtime.

### Keys
#### Configuration  
Configuration is a map that defines either multiple configuration options, or specifies the default location of the
configuration file. If a map has multiple keys, those keys are treated as the configuration keys in the package map
used in the SVSM configuration. 

Such a map may define a location key and an 'template_location' key. The latter will be copied to the location provided,
which may then be modified by the user by default.

#### Name  
Name defines the name to use with XBPS to install the package. While it may be omitted (doing so will result in SVSM
attempting to use the symbol name to install it), it is recommended to include it for completeness.


#### is_nonfree
This is a simple boolean, defining whether the non-free repository package needs to be installed and used. By
default, this is false.

#### is_restricted
This is a simple boolean that defines whether the package is `restricted` by XBPS or not. If a package is restricted
then we *must* use a local void-packages repo and generate the package locally, and if we don't an error should be
issued.

## At Run-Time
When SVSM is run, it first creates the environment as normal, and then creates the packages and puts them into the
environment as variables when necessary. Part of this step involves doing a `xbps-query` to get package information and
the like. Afterwards, SVSM parses these files in %DIRECTORY% and then replaces the definitions built-in.

(As for why a pvsm package takes precedence over built-in definitions, it is because pvsm file can generally be assumed
to be more up-to-date than svsm's built-ins as it is more flexible.)

## Alternative Solutions
#### Alternative Package Definition Format in SVSM
An alternative to package files that has some potential, is to instead include some of this in the default SVSM system
definitions by default (For example have the user define that bash has configuration files bashrc and bash_profile and
where to put them). The solution would create a file that potentially looks like this:


```nix
system.config = {
    services = [{name = sshd},];

    vp_repos = {
        personal = {
            location = (vp-r 'sapein');
            branch = 'personal';
            allow_restricted = true;
        };
    };
    
    users = {
        sapeint = {
            hashedPassword = 'PASS';
            homedir = {
                subdirs = [ ./library, ./games/launchers, ./develop/personal, ./writing, ./videos, ./ttrpg, ];
            };
            dotfiles = gh-r 'sapein' 'dotfiles'; # This will be used, but partially overridden
            packages = [ 
                i3status, i3lock, dmenu, firefox, nerdfont, zathura, zathura-pdf-mupdf, ctags, socat, krita, telegram-desktop, pulseaudio,
                pipewire, pavucontrol, helvum, xdg-user-dirs, feh, git-lfs, wireplumber, steam, stalonetray, 
                i3 {
                    config = {
                        config_location = (home ./.config/i3/config) ;
                        file = use_file ./i3/config (gh-r 'sapein' 'dotfiles');
                    };
                },
                discord { repository = 'personal';},
                st { repository = 'personal'; },
                dunst { config =  {
                            config_location = (home ./.config/dunst/dunstrc);
                            file = use_file ./dunst/dunstrc (gh-r 'sapein' 'dotfiles');
                        };
                },
                bash {
                    config = {
                        bashrc = {
                            config_location = (home ./.bashrc);
                            file = use_file ./bash/bashrc (gh-r 'sapein' 'dotfiles');
                        };
                        bash_profile = {
                            config_location = (home ./.bash_profile);
                            file = use_file ./bash/bash_profile (gh-r 'sapein' 'dotfiles');
                        };
                    };
                },
                xorg {
                    config = {
                        config_location = (home ./.xinitrc);
                        file = add_lines [ './screenlayout/dualmonitor.sh', 'dbus-run-session i3',];
                    };
                },
            ];
        };
    };
};
```

Aside from the fact that this is very lengthy, it also requires the user to define explicitly where to put files, which
may be nice, but requires the user know exactly how the package works and also know exactly what incantations to put.
This may lead to issues where someone forgets to define where the file originates from, leading to an error that may be
hard or otherwise difficult to track down for an inexperienced end-user. 

Furthermore, this increases the length of a system definition file and means that the user must now micromanage the
packages if they wish to change or update it. This is somewhat antithetical to the idea/goal of SVSM. This is not to say
that this method won't be implemented into SVSM for users that wish to do so, however, we need a way to handle this for
most users. 

Additionally, this merely solves the configuration issue, not the issue of package origin, so restricted packages might
cause some confusion to new users. While it can be expected that a void user would understand how restricted packages
work, and thus expect to add that in as a repository source, not knowing if it's restricted or not means we can not give
as meaningful errors to the end user if we did not require all restricted packages to be partially built-ins. Which
leads us to the issue of bloating SVSM and requiring updates for any changes or new packages. Where-as with package
files a user can define a new package, even if it's not in void-packages, allowing them to use other people's packages
that aren't mainline.


#### Creating different syntax for packages in SVSM/SVSL  
Packages could be defined in a different syntax, however, this could cause issues, and is a bit of a departure from
the rest of the language as the language attempts to avoid things like special definitions and the like.

#### Permitting Function Definitions  
This is -- broadly -- a non-starter.  As I've designed SVSL to be a pure language, and I've deliberately omitted
functions, adding them just to solve this singular issue seems impractical at best, and at worst overkill. Furthermore,
I would still need to ultimately design it to be handled like this and make that syntax valid in any SVSL file. Which
I'm not particularly inclined to do as this time.

#### Creating specific-use syntax  
This would be useful, however the syntax would both be similar to -- if not the same as -- the result from this. The
only difference is an additional-reserved keyword. While this isn't such a bad thing, the ultimate outcome would not be
much different overall, but would require additional work in supporting it in the parser and interpreter. This would
come with a lot of extra work. Where-as this only requires I write the generation functions for handling this and I can
reuse most of what I have. I can then later implement this in the future.

#### Getting XBPS to handle this more 'properly'  
While it would be nice if XBPS handled things differently, or in such a way to make this easier, it is unfortunately
not entirely possible. SVSM is a relatively small project that doesn't even work, nor has it shown to have any interest
in the wider Void Linux community. In the future, if SVSM caught on or became 'official', then it would be possible for
such a thing to occur. However, given our niche status it might not entirely work out.

#### Inspecting package templates/packages  
This is both hacky, and prone to issues. For example bash does *not* declare the bashrc or bash_profile information
anywhere in its template, and also it would *require* us to always get a copy of the repo when doing things, which may
take up space we don't need to take up.



## Examples
#### Bash
```nix
# bash.pvsm

bash = {
    configuration = { 
        bashrc = { location = (home ./.bashrc); },
        bash_profile = { location = (home ./.bash_profile); },
    };
    name = 'bash' ; 
}
```

#### Discord
```nix
# discord.pvsm

discord = {
    name = 'Discord' ;
    is_restricted = true ;
}
```
