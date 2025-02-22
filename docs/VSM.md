# Void System Manager (VSM)
    Void System Manager is an attempt to bring some of the nice ‘reproducibility’ of NixOS to Void Linux without directly changing Void Linux.


## VSM Command Line  
vsm | void-system-manager (opts) (command)

**Commands**:
```
  deploy [github url]
  install [pkg] ([config])
  remove [pkg]
  configure [pkg] ([config])
  list-pkgs
  freeze-pkgs
  pin-pkg [pkg]

  add-source [url] ([name])
  remove-source ([url]|[name])
  list-source
    
  add-vpks [github url] ([name])
  remove-vpks [github url] ([name])

  enable-service [service name|pkg]
  disable-service [service name|pkg]
```

**Options**:
```
  --config_location | -c [location]
  --state_location | -s [location]
```

## VSM Files  
### Configuration  
VSM Configuration is generally done through the ‘config.vsm’ file. This is
similar to the configuration.nix file found on most nix installs, the
purpose of which is to track the current ‘main’ state of the system. 

The language in VSM is VSL (Void System Language) a DSL created for this
project.

#### Design Considerations
VSL is designed as a very restricted DSL that focuses on being entirely
declarative. To that end, VSL does not have a built-in method of defining
functions or permitting side effects, outside of builtins provided by the
language. Furthermore, usual programming operations are heavily restricted to
that end. Additionally, the language is intended to be kept simple so that it’s
easy to create alternative implementations. 

#### Grammar
##### Syntax

```ebnf
stmt     := expr* comment*;
expr     := literal | fncall | map | list | path | listref | mapref | vardec;
expr     := '(' expr* ')';
expr     := 'import' path;

comment  := "#" .*;

vardec   := (SYMBOL | mapref | listref) '=' expr;

path     := pathabs | pathrel;
pathabs  :=  '/' ([^\s] | STRING | pathabs)*;
pathrel  := './' ([^\s] | STRING | pathabs)*;

fncall   := "FNAME" expr*;

map      := '{' mapattr* '}';
mapattr  := "SYMBOL" '=' expr ';';

list     := '[' listattr* ']';
listattr := expr ',';

mapref   := "SYMBOL" '.' "VAL";
listref  := "NAME" '[' "NUMBER" ']';

literal  := "NUMBER" | "STRING" | "BOOL" | "SYMBOL";
```

##### Lexical
```
STRING := '"' [ (. - '"' )* ] '"'
STRING := "'" [ (. - "'" )* ] "'"
BOOL   := "true" | "false"
NUMBER := DIGIT+ ( "." DIGIT+ )?
SYMBOL := ALPHA+ ( ALPHA | DIGIT | "-" )*
ALPHA  := "a" ... "z" | "A" ... "Z" | "_";
DIGIT  := "0" ... "9";
```


#### Data Types
The data-types in VSL are as follows:

##### Strings
A string is defined as any visible character, plus whitespace, between either
two double quotes (“) or single quotes (‘) with no way to escape, although it
does accept newlines. This is in accordance with the STRING lexical rules.

##### Numbers
A Number is any number greater than or equal to zero.

##### Boolean
A boolean represents True or False.

##### Functions
Functions are not definable in VSL, they are only provided by the
implementation. This is a deliberate design decision, as VSL is intended to be
entirely declarative, and it also simplifies the implementation. 

##### Map
A map is a dictionary, in that it's elements are accessible through a symbol.
A map is equivalent to 'attr set' in Nix.

##### List
A list is a list.

##### Path
A path is a file system path.

##### Package
A void package

##### Package Repository
A void package repository

##### Void Package Repository (Subclass of Git Repo)
A clone of the void packages repository

##### Git Repo.
A Git Repository.

##### Symbol
A symbol is similar to a string, but in that it is an indivisible unit. They may refer to themselves,
function names, etc.

##### Option
This type is purely for 'function arguments'. An Option<T> just means that the argument can be omitted.

#### Builtins
##### Functions
As VSL/VSM does not have an inbuilt function definition or declaration syntax, this document uses the following syntax for function declaration:

`fn [name] [arg1](:[type1]), [arg2], ..., [argN] (-> [ReturnType])`

`github-repo user: string, repo: string -> string`
`gh-r user: string, repo: string -> string`


`voidpackages-repo, user: string`
`vp-r user: string`

`add_line line: string`

`home path_relative -> Path`


`replace original: string, replacement: string, string: string -> string`


`remove original: string, replacement: string`


`use_file to_use: path, repo: Option<repo>`


`join char: string, strings: [string] -> string`

`print string: [string]`



Example
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
                    config = use_file ./i3/config (gh-r 'sapein' 'dotfiles');
                },
                discord { repository = 'personal';},
                st { repository = 'personal'; },
                dunst { config = use_file ./dunst/dunstrc (gh-r 'sapein' 'dotfiles'); },
                bash {
                    bashrc = use_file ./bash/bashrc (gh-r 'sapein' 'dotfiles');
                    bash_profile = use_file ./bash/bash_profile (gh-r 'sapein' 'dotfiles'); 
                },
                xorg {
                    config = add_lines [ './screenlayout/dualmonitor.sh', 'dbus-run-session i3',];
                },
            ];
        };
    };
};
```
