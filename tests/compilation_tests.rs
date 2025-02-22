use svsm::interpreter::Interpreter;
use svsm::lex::Lexer;
use svsm::parser::Parser;

#[test]
#[ignore = "This doesn't actually work yet."]
fn test_full() {
    let input_str = "
    system.config = {
        services = [{name = sshd}];

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
    };";

    let mut lexer = Lexer::from_string(input_str);

    let lexer_output = lexer.tokenize_input_smart();

    let mut parser = Parser::from_token_list_smart(lexer_output);
    let parsed_output = parser.parse_input();
    let mut interpriter = Interpreter::new(parsed_output.clone()).create_standard_env();
    let output = interpriter.eval();
    println!("{:#?}", output);
}
