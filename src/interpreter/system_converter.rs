use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use crate::system::{System, Service, PackageRepository, Source, User, RemoteSource, HomeDirectory, Package};
use crate::parser::Expr;
use crate::system::RemoteSource::VoidRepo;

impl System {
    pub fn from_map(map: Expr) -> Self{
        System {
            services: HashMap::from_iter(
                map.get_map_value(Expr::symbol_from_str("services"))
                    .into_iter()
                    .flat_map(Service::from_list)
                    .collect::<Vec<(Rc<str>, Service)>>()
            ),
            repositories: HashMap::from_iter(
                map
                    .get_map_value(Expr::symbol_from_str("vp_repos"))
                    .into_iter()
                    .flat_map(PackageRepository::from_big_map)
                    .collect::<Vec<(Rc<str>, PackageRepository)>>()
            ),
            users: HashMap::new(),
            system_packages: Rc::from(""),
        }
    }
}


impl Service {
    pub fn from_list(list: &Expr) -> Vec<(Rc<str>, Self)> {
        let list = match list {
            Expr::List(list) => list,
            _ => panic!("Unable convert non-list type to services!"),
        };
        list.iter().map(Service::from_map).collect::<Vec<(Rc<str>, Self)>>()
    }
    pub fn from_map(map: &Expr) -> (Rc<str>, Service) {
        let name = match map.get_map_value(Expr::symbol_from_str("name")) {
            Some(Expr::String(str)) => str.clone(),
            _ => panic!("Name must be provided!"),
        };

        (name.clone(), Service {
            name,
            enabled: match map.get_map_value(Expr::symbol_from_str("enabled")) {
                Some(Expr::Boolean(bool)) => *bool,
                _ => true,
            },
            downed: match map.get_map_value(Expr::symbol_from_str("downed")) {
                Some(Expr::Boolean(bool)) => *bool,
                _ => false,
            },
        })
    }
}

impl PackageRepository {
    fn from_big_map(map: &Expr) -> Vec<(Rc<str>, PackageRepository)> {
        let map = match map {
            Expr::Map(map) => map,
            _ => panic!("Unable to convert non-map type to package repositories!"),
        };

        map
            .iter()
            .map(|(n, m)| {(n.clone().extract_str(), PackageRepository::from_map(m, n))})
            .collect::<Vec<(Rc<str>, PackageRepository)>>()
    }

    pub fn from_map(map: &Expr, name: &Expr) -> PackageRepository {
        println!("{:?}", name);
        println!("{:?}", map);
        let name = match name {
            Expr::Symbol(rc) => rc,
            _ => panic!("Name is not a symbol!"),
        };

        let location = match map.get_map_value(Expr::symbol_from_str("location")) {
            Some(Expr::GitHubRemote { user, repo, branch}) => {
                Source::Remote(RemoteSource::GithubRemote {
                    user: user.clone(),
                    repository_name: repo.clone(),
                    branch_name: match branch {
                        Some(b) => Some(b.clone()),
                        None => match map.get_map_value(Expr::symbol_from_str("branch")) {
                            Some(Expr::String(str)) => Some(str.clone()),
                            _ => None
                        },
                    },
                })
            },
            Some(Expr::String(_)) => todo!(),
            _ => panic!("system.config.vp_repos.{repo}.location is not a valid type or was not in the map!", repo=name),
        };

        PackageRepository {
            name: Some(name.to_owned()),

            location,
            allow_restricted: match map.get_map_value(Expr::symbol_from_str("allow_restricted")) {
                Some(Expr::Boolean(allow)) => *allow,
                _ => false,
            },
        }
    }
}

impl User {
    pub fn from_big_map(map: &Expr) -> Vec<(Rc<str>, User)>{
        let map = match map {
            Expr::Map(map) => map,
            _ => panic!("Unable to convert non-map type to package repositories!"),
        };
        
        map
            .iter()
            .map(|(n, m)| {(n.clone().extract_str(), User::from_map(m, n))})
            .collect::<Vec<(Rc<str>, User)>>()
    }

    pub fn from_map(map: &Expr, name: &Expr) -> User {
        let username = match name {
            Expr::Symbol(rc) => rc,
            _ => panic!("Name is not a symbol!"),
        };
        
        let homedir = match map.get_map_value(Expr::symbol_from_str("homedir")) {
            Some(Expr::Map(map)) => {
                HomeDirectory::Path {
                    location: if let Some((_, location)) = map.get_key_value(&Expr::Symbol(Rc::from("location"))) {
                        PathBuf::from(location.clone().extract_str().to_string())
                    } else {
                        let mut path = PathBuf::from("/home/");
                        path.push(username.to_string());
                        path
                    },
                    subdirs: if let Some((_, subdirs)) = map.get_key_value(&Expr::Symbol(Rc::from("subdirs"))) {
                        match subdirs {
                            Expr::List(list) => list.into_iter().map(|e| {
                                match e {
                                    Expr::Path(path) => path.to_owned(),
                                    _ => panic!("Only Paths are allowed in subdirs!"),
                                }
                            }).collect(),
                            _ => panic!("Subdirs must be a list, or be missing"),
                        }
                    } else {
                        vec![]
                    },
                }
            }
            Some(Expr::String(_)) => todo!(),
            None => HomeDirectory::Path {
                location: {
                    let mut path = PathBuf::from("/home/");
                    path.push(username.to_string());
                    path
                },
                subdirs: vec![],
            },
            
            _ => panic!("system.config.users.{username}.homedir is not a valid type!", username=username),
        };
        
        let dotfiles = match map.get_map_value(Expr::symbol_from_str("dotfiles")) {
            Some(Expr::GitHubRemote { user, repo, branch }) => {
                Some(Source::Remote(RemoteSource::GithubRemote {
                    user: user.to_owned(),
                    repository_name: repo.to_owned(),
                    branch_name: branch.to_owned(),
                }))
            },
            None => None,
            _ => panic!("system.config.users.{username}.dotfiles is not a valid type!", username = username),
        };
        
        let packages = match map.get_map_value(Expr::symbol_from_str("packages")) {
            Some(Expr::List(list)) => {
                let map = list.iter()
                    .map(|e| {
                        match e {
                            Expr::Symbol(name) => (name.to_owned(), Package {
                                config: None,
                                repository: Source::Remote(VoidRepo),
                            }),
                            _ => panic!("Unknown Expr when handling packages."),
                        }
                    });
                HashMap::from_iter(map)
            }
            None => HashMap::new(),
            _ => panic!("{:?}", map.get_map_value(Expr::Symbol(Rc::from("packages"))))
        };
        
        User {
            username: Some(username.to_owned()),
            homedir,
            dotfiles,
            packages,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use crate::system::{HomeDirectory, Source};
    use super::*;
    #[test]
    fn test_service_from_map() {
        let map = Expr::Map(BTreeMap::from([
            (Expr::Symbol(Rc::from("name")), Expr::String(Rc::from("test"))),
        ]));
        let expected = (Rc::from("test"), Service {
            name: Rc::from("test"),
            enabled: true,
            downed: false,
        });

        assert_eq!(Service::from_map(&map), expected);
    }

    #[test]
    fn test_service_from_list() {
        let list = Expr::List(vec![
            Expr::Map(BTreeMap::from([
                (Expr::Symbol(Rc::from("name")), Expr::String(Rc::from("test"))),
            ])),
            Expr::Map(BTreeMap::from([
                (Expr::Symbol(Rc::from("name")), Expr::String(Rc::from("test2"))),
                (Expr::Symbol(Rc::from("enabled")), Expr::Boolean(true)),
                (Expr::Symbol(Rc::from("downed")), Expr::Boolean(true)),
            ])),
        ]);
        let expected = vec![
            (Rc::from("test"),
             Service {
                 name: Rc::from("test"),
                 enabled: true,
                 downed: false,
             }
            ),
            (Rc::from("test2"),
             Service {
                name: Rc::from("test2"),
                enabled: true,
                downed: true,
             }
            ),
        ];

        assert_eq!(Service::from_list(&list), expected);
    }

    #[test]
    fn test_package_repository_from_big_map() {
        let map = Expr::Map(BTreeMap::from([
            (Expr::symbol_from_str("personal"),
             Expr::Map(BTreeMap::from([
                 (Expr::symbol_from_str("location"),
                  Expr::GitHubRemote {
                      user: Rc::from("sapein"),
                      repo: Rc::from("void-packages"),
                      branch: None,
                  }),
                 (Expr::symbol_from_str("branch"),
                  Expr::string_from_str("personal")),
                 (Expr::symbol_from_str("allow_restricted"),
                  Expr::Boolean(true))
             ])))
        ]));

        let expected = vec![(Rc::from("personal"), PackageRepository {
            name: Some(Rc::from("personal")),
            location: Source::Remote(RemoteSource::GithubRemote {
                user: Rc::from("sapein"),
                repository_name: Rc::from("void-packages"),
                branch_name: Some(Rc::from("personal")),
            }),
            allow_restricted: true,
        })];

        assert_eq!(PackageRepository::from_big_map(&map), expected);
    }

    #[test]
    fn test_package_repository_from_map() {
        let map = Expr::Map(BTreeMap::from([
            (Expr::symbol_from_str("location"),
             Expr::GitHubRemote {
                 user: Rc::from("sapein"),
                 repo: Rc::from("void-packages"),
                 branch: None,
             }),
            (Expr::symbol_from_str("branch"),
             Expr::string_from_str("personal")),
            (Expr::symbol_from_str("allow_restricted"),
             Expr::Boolean(true))
        ]));

        let expected = PackageRepository {
            name: Some(Rc::from("personal")),
            location: Source::Remote(RemoteSource::GithubRemote {
                user: Rc::from("sapein"),
                repository_name: Rc::from("void-packages"),
                branch_name: Some(Rc::from("personal")),
            }),
            allow_restricted: true,
        };

        assert_eq!(PackageRepository::from_map(&map, &Expr::symbol_from_str("personal")), expected);
    }

    #[test]
    fn test_users_from_big_map() {
        let map = Expr::Map(BTreeMap::from([
            (Expr::symbol_from_str("sapeint"),
             Expr::Map(
                 BTreeMap::from([
                     (Expr::symbol_from_str("hashedPassword"),
                      Expr::string_from_str("PASS")),
                     (Expr::symbol_from_str("homedir"),
                      Expr::Map(BTreeMap::from([
                          (Expr::symbol_from_str("subdirs"),
                           Expr::List(vec![
                               Expr::Path(PathBuf::from("./library")),
                               Expr::Path(PathBuf::from("./games/launchers")),
                           ]))
                      ]))),
                     (Expr::symbol_from_str("dotfiles"),
                      Expr::GitHubRemote {
                          user: Rc::from("sapein"),
                          repo: Rc::from("dotfiles"),
                          branch: None,
                      }),
                     (Expr::symbol_from_str("packages"),
                      Expr::List(vec![
                          Expr::symbol_from_str("dmenu"),
                          Expr::symbol_from_str("firefox"),
                      ]))
                 ])))
        ]));

        let expected = vec![
            (Rc::from("sapeint"),
             User {
                 username: Some(Rc::from("sapeint")),
                 homedir: HomeDirectory::Path {
                     location: PathBuf::from("/home/sapeint"),
                     subdirs: vec![PathBuf::from("./library"), PathBuf::from("./games/launchers")],
                 },
                 dotfiles: Some(Source::Remote(RemoteSource::GithubRemote {
                     user: Rc::from("sapein"),
                     repository_name: Rc::from("dotfiles"),
                     branch_name: None,
                 })),
                 packages: HashMap::from([
                     (Rc::from("firefox"), crate::system::Package {
                         config: Default::default(),
                         repository: Source::Remote(RemoteSource::VoidRepo)
                     }),
                     (Rc::from("dmenu"), crate::system::Package {
                         config: Default::default(),
                         repository: Source::Remote(RemoteSource::VoidRepo)
                     }),
                 ]),
             }
            )
        ];

        assert_eq!(User::from_big_map(&map), expected);
    }

    #[test]
    fn test_users_from_map() {
        let map =
            Expr::Map(BTreeMap::from([
                (Expr::symbol_from_str("hashedPassword"),
                 Expr::string_from_str("PASS")),
                (Expr::symbol_from_str("homedir"),
                 Expr::Map(BTreeMap::from([
                     (Expr::symbol_from_str("subdirs"),
                      Expr::List(vec![
                          Expr::Path(PathBuf::from("./library")),
                      ]))
                 ]))),
                (Expr::symbol_from_str("dotfiles"),
                 Expr::GitHubRemote {
                     user: Rc::from("sapein"),
                     repo: Rc::from("dotfiles"),
                     branch: None,
                 }),
                (Expr::symbol_from_str("packages"),
                 Expr::List(vec![
                     Expr::symbol_from_str("dmenu"),
                     Expr::symbol_from_str("firefox"),
                 ]))
            ]));

        let expected = User {
            username: Some(Rc::from("sapeint")),
            homedir: HomeDirectory::Path {
                location: PathBuf::from("/home/sapeint"),
                subdirs: vec![PathBuf::from("./library")],
            },
            dotfiles: Some(Source::Remote(RemoteSource::GithubRemote {
                user: Rc::from("sapein"),
                repository_name: Rc::from("dotfiles"),
                branch_name: None,
            })),
            packages: HashMap::from([
                (Rc::from("firefox"), crate::system::Package {
                    config: Default::default(),
                    repository: Source::Remote(RemoteSource::VoidRepo)
                }),
                (Rc::from("dmenu"), crate::system::Package {
                    config: Default::default(),
                    repository: Source::Remote(RemoteSource::VoidRepo)
                }),
            ]),
        };

        assert_eq!(User::from_map(&map, &Expr::symbol_from_str("sapeint")), expected);
    }

    #[test]
    fn test_system_from_map() {
        let map = Expr::Map(BTreeMap::from([
            (Expr::Symbol(Rc::from("services")),
             Expr::List(vec![
                 Expr::Map(BTreeMap::from([
                     (Expr::Symbol(Rc::from("name")), Expr::String(Rc::from("test"))),
                     (Expr::Symbol(Rc::from("enabled")), Expr::Boolean(false))
                 ]))
            ])),
            (Expr::Symbol(Rc::from("vp_repos")),
             Expr::Map(BTreeMap::from([
                 (Expr::Symbol(Rc::from("test")),
                  Expr::Map(BTreeMap::from([
                      (Expr::Symbol(Rc::from("location")),
                       Expr::GitHubRemote {
                           user: Rc::from("void"),
                           repo: Rc::from("void-packages"),
                           branch: None,
                       }),
                  ])))
             ]))),
        ]));

        let expected = System {
            services: HashMap::from([
                (Rc::from("test"), Service {
                    name: Rc::from("test"),
                    enabled: false,
                    downed: false,
                })
            ]),
            repositories: HashMap::from([
                (Rc::from("test"),
                 PackageRepository {
                     name: Some(Rc::from("test")),
                     location: Source::Remote(RemoteSource::GithubRemote {
                         user: Rc::from("void"),
                         repository_name: Rc::from("void-packages"),
                         branch_name: None,
                     }),
                     allow_restricted: false,
                 })
            ]),
            users: HashMap::new(),
            system_packages: Rc::from(""),
        };

        assert_eq!(System::from_map(map), expected);
    }
}