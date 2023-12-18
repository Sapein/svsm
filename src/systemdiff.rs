use crate::system::{System, User};
use UserDiffer::{UserDiff, diff_users};
#[derive(Debug, PartialEq)]
struct SystemDiff {
    users: Vec<UserDiff>
}

impl SystemDiff {
    pub fn diff(start: System, end: System) -> Self{
        Self {
            users: diff_users(start.users, end.users),
        }
    }
}

mod UserDiffer {
    use crate::system::User;

    #[derive(Debug, PartialEq)]
    pub(crate) enum UserDiff {
        Alter(User),
        Remove(User),
        Add(User)
    }

    impl User {
        pub(crate) fn diff_user(&self, other: Option<&Self>) -> Option<UserDiff> {
            match other {
                Some(T) if self == T && self.is_different(T) => Some(UserDiff::Alter(T.clone())),
                _ => None
            }
        }
    }

    pub(crate) fn diff_users(start: Vec<User>, end: Vec<User>) -> Vec<UserDiff> {
        let user_remove = start.iter().filter_map(|u| {
            if !end.contains(&u) {
                Some(UserDiff::Remove(u.clone()))
            } else {
                None
            }
        });

        let user_add = end.iter().filter_map(|u| {
            if !start.contains(&u) {
                Some(UserDiff::Add(u.clone()))
            } else {
                start.iter().find(|&iu| u == iu).unwrap().diff_user(Some(u))
            }
        });

        let mut result = user_remove.collect::<Vec<UserDiff>>();
        result.extend(user_add);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_user_diff_none() {
        let start = User::new(&String::from("test"));
        let end = User::new(&String::from("other"));
        let result = start.diff_user(Some(&end));

        assert!(result.is_none())
    }

    #[test]
    pub fn test_user_diff() {
        let start = User::new(&String::from("test"));
        let mut end = User::new(&String::from("other"));

        end.name = start.name.clone();
        end.hashed_password = start.hashed_password.clone();
        end.dotfiles = start.dotfiles.clone();

        let end = end;

        let result = start.diff_user(Some(&end));

        assert_eq!(result, Some(UserDiff::Alter(end)));
    }

    #[test]
    pub fn test_system_user_diff() {
        let start = System::new().add_user(User::new(&String::from("test")));
        let end = System::new().add_user(User::new(&String::from("other")));
        let result = SystemDiff::diff(start, end);
        let output = SystemDiff {
            users: vec![UserDiff::Remove(User::new(&String::from("test"))), UserDiff::Add(User::new(&String::from("other")))]
        };

        assert_eq!(result, output);
    }

    #[test]
    #[should_panic]
    pub fn panic_two_plus_two() {
        assert_eq!(2 + 2, 5)
    }
}
