use std::rc::Rc;
use crate::parser::Expr;
use crate::system::*;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Action {
    File(FileSystemAction),
    System(SystemAction),
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
/// Represents Actions we perform on the system as a whole.
pub enum SystemAction {
    AddPackage {
        package_name: String,
        package_repository: PackageRepository
    },
    
    RemovePackage {
        package_name: String,
    },
    
    AddRepository {
        package_repository: PackageRepository
    },
    
    RemoveRepository {
        package_repository: PackageRepository
    },
    
    ConfigurePackage {
        package_name: String,
        configuration_actions: Vec<Action>,
    },
}

/// Represents an action we can perform on the File System
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum FileSystemAction {
    MoveFile {
        original_location: Rc<Expr>,
        final_location: Rc<Expr>,
        is_dir: bool,
    },

    CopyFile {
        original_location: Rc<Expr>,
        final_location: Rc<Expr>,
        is_recursive: bool,
    },

    RenameFile {
        original_name: Rc<Expr>,
        final_name: Rc<Expr>,
    },

    AddToFile {
        original_file: Rc<Expr>,
        content_to_add: Rc<Expr>,
    },

    RemoveFile {
        file_location: Rc<Expr>,
        is_dir: bool,
    },

    CreateFile {
        file_location: Rc<Expr>,
        contents: Option<Rc<Expr>>,
        is_dir: bool,
    },
}