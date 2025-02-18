use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;


/// This is designed to represent the world/system in SVSM.
/// This does not represent the base level `system` map, but
/// represents the `system.config` and `system.current` values.
#[derive(Debug, PartialEq)]
pub(crate) struct System {
    pub(crate) services: HashMap<Rc<str>, Service>,
    pub(crate) repositories: HashMap<Rc<str>, PackageRepository>,
    pub(crate) users: HashMap<Rc<str>, User>,
    pub(crate) system_packages: Rc<str> //TODO: replace with actual data.
}

#[derive(Debug, PartialEq)]
pub(crate) struct Service {
    pub(crate) name: Rc<str>,
    pub(crate) enabled: bool,
    pub(crate) downed: bool
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Ord, PartialOrd)]
pub(crate) struct PackageRepository {
    pub(crate) name: Option<Rc<str>>,
    pub(crate) location: Source,
    pub(crate) allow_restricted: bool,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum Source {
    Remote(RemoteSource),
    Local(LocalSource)
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum RemoteSource {
    GithubRemote {
        user: Rc<str>,
        repository_name: Rc<str>,
        branch_name: Option<Rc<str>>,
    },
    GitRemote {
        url: Rc<str>,
        branch_name: Option<Rc<str>>,
    },
    VoidRemote(Rc<str>),
    VoidRepo
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum LocalSource {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Debug, PartialEq)]
pub(crate) struct User {
    pub(crate) username: Option<Rc<str>>,
    pub(crate) homedir: HomeDirectory,
    pub(crate) dotfiles: Option<Source>,
    pub(crate) packages: HashMap<Rc<str>, Package>,

}

#[derive(Debug, PartialEq)]
pub(crate) enum HomeDirectory {
    Path {
        location: PathBuf,
        subdirs: Vec<PathBuf>,
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Package {
    pub(crate) config: Option<PathBuf>,
    pub(crate) repository: Source,
}