use std::path::PathBuf;
use std::rc::Rc;

pub(crate) struct System {
    pub(crate) users: Vec<User>,
}

impl System {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
        }
    }
    pub fn add_user(self, user: User) -> Self {
        let mut users = self.users;
        users.push(user);

        Self {
            users,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub name: Rc<str>,
    pub homedir: PathBuf,

    pub hashed_password: Option<Rc<str>>,

    pub dotfiles: Option<FileSource>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl User {
    pub fn is_different(&self, other: &Self) -> bool {
        ! (self.name == other.name &&
            self.homedir == other.homedir &&
            self.hashed_password == other.hashed_password &&
            self.dotfiles == other.dotfiles)
    }
}



impl User {
    pub fn new(name: &String) -> Self {
        Self {
            name: Rc::from(name.clone()),
            homedir: PathBuf::from(String::from("/home/") + name),
            hashed_password: None,
            dotfiles: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileSource {
    GitHub(GithubRepo),
    Local(PathBuf),
    Remote(Rc<str>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GithubRepo {
    user: Rc<str>,
    repo_name: Rc<str>,
    branch: Option<Rc<str>>,
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn two_plus_two() {
        assert_eq!(2+2, 4)
    }

    #[test]
    #[should_panic]
    pub fn panic_two_plus_two() {
        assert_eq!(2 + 2, 5)
    }
}
