//! Bind evidence to a store and use an atomic version check at the effect.
use ghostproof::{Named, name};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UserId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProjectId(u64);

mod policy {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, PartialEq, Eq)]
    pub enum Error {
        Denied,
        Missing,
        Conflict,
        Backend,
    }

    struct Row {
        project: ProjectId,
        owner: UserId,
        version: u64,
        name: String,
    }

    /// A shared backend with mutable state.
    #[derive(Clone)]
    pub struct Store(Arc<Mutex<Row>>);

    impl Store {
        pub fn new() -> Self {
            Self(Arc::new(Mutex::new(Row {
                project: ProjectId(7),
                owner: UserId(1),
                version: 0,
                name: "before".into(),
            })))
        }

        /// Simulate a separately authorized administrative change.
        /// Every change relevant to the checked fact increments this revision.
        pub fn change_owner(&self, owner: UserId) -> Result<(), Error> {
            let mut row = self.0.lock().map_err(|_| Error::Backend)?;
            let next = row.version.checked_add(1).ok_or(Error::Backend)?;
            row.owner = owner;
            row.version = next;
            Ok(())
        }

        pub fn current_name(&self) -> Result<String, Error> {
            Ok(self.0.lock().map_err(|_| Error::Backend)?.name.clone())
        }
    }

    #[ghostproof::proof(subjects(store, actor, project))]
    pub struct CanRename<'store, 'user, 'project> {
        version: u64,
    }

    pub fn check<'s, 'u, 'p>(
        store: &Named<'s, Store>,
        user: &Named<'u, UserId>,
        project: &Named<'p, ProjectId>,
    ) -> Result<CanRename<'s, 'u, 'p>, Error> {
        let row = store.value().0.lock().map_err(|_| Error::Backend)?;
        if row.project != *project.value() {
            return Err(Error::Missing);
        }
        if row.owner != *user.value() {
            return Err(Error::Denied);
        }
        Ok(CanRename::issue(store, user, project, row.version))
    }

    impl<'s, 'u, 'p> CanRenameCapability<'s, 'u, 'p, Store, UserId, ProjectId> {
        /// Consume the permission, validating freshness and writing atomically.
        pub fn rename(self, name: &str) -> Result<(), Error> {
            let mut row = self.store().value().0.lock().map_err(|_| Error::Backend)?;
            if row.project != *self.project().value() {
                return Err(Error::Missing);
            }
            if row.version != self.proof().version {
                return Err(Error::Conflict);
            }
            let next = row.version.checked_add(1).ok_or(Error::Backend)?;
            // The same lock covers the revision comparison and the effect.
            row.name = name.into();
            row.version = next;
            Ok(())
        }
    }
}

fn main() {
    let backend = policy::Store::new();
    name!(
        store = backend.clone(),
        user = UserId(1),
        project = ProjectId(7)
    );
    let permission = policy::check(&store, &user, &project).unwrap();
    backend.change_owner(UserId(2)).unwrap();
    let permission = permission.bind(store, user, project);
    assert_eq!(
        permission.rename("stale write"),
        Err(policy::Error::Conflict)
    );
    assert_eq!(backend.current_name().unwrap(), "before");

    name!(
        store = backend.clone(),
        user = UserId(2),
        project = ProjectId(7)
    );
    let permission = policy::check(&store, &user, &project).unwrap();
    permission
        .bind(store, user, project)
        .rename("after")
        .unwrap();
    assert_eq!(backend.current_name().unwrap(), "after");
}

#[cfg(test)]
mod tests {
    use super::*;
    use policy::{Error, Store};

    #[test]
    fn current_permission_writes_and_rechecks_the_new_revision() {
        let backend = Store::new();
        name!(
            store = backend.clone(),
            user = UserId(1),
            project = ProjectId(7)
        );
        policy::check(&store, &user, &project)
            .unwrap()
            .bind(store, user, project)
            .rename("first")
            .unwrap();
        assert_eq!(backend.current_name().unwrap(), "first");
        name!(
            store = backend.clone(),
            user = UserId(1),
            project = ProjectId(7)
        );
        policy::check(&store, &user, &project)
            .unwrap()
            .bind(store, user, project)
            .rename("second")
            .unwrap();
        assert_eq!(backend.current_name().unwrap(), "second");
    }

    #[test]
    fn revoked_permission_cannot_write_and_new_check_is_denied() {
        let backend = Store::new();
        name!(
            store = backend.clone(),
            user = UserId(1),
            project = ProjectId(7)
        );
        let permission = policy::check(&store, &user, &project).unwrap();
        backend.change_owner(UserId(2)).unwrap();
        assert!(matches!(
            policy::check(&store, &user, &project),
            Err(Error::Denied)
        ));
        assert_eq!(
            permission.bind(store, user, project).rename("unauthorized"),
            Err(Error::Conflict)
        );
        assert_eq!(backend.current_name().unwrap(), "before");
    }

    #[test]
    fn earlier_revision_stays_stale_after_owner_changes_back() {
        let backend = Store::new();
        name!(
            store = backend.clone(),
            user = UserId(1),
            project = ProjectId(7)
        );
        let permission = policy::check(&store, &user, &project).unwrap();
        backend.change_owner(UserId(2)).unwrap();
        backend.change_owner(UserId(1)).unwrap();
        assert!(policy::check(&store, &user, &project).is_ok());
        assert_eq!(
            permission.bind(store, user, project).rename("old revision"),
            Err(Error::Conflict)
        );
        assert_eq!(backend.current_name().unwrap(), "before");
    }

    #[test]
    fn missing_resource_and_denial_are_distinct() {
        name!(
            store = Store::new(),
            user = UserId(2),
            project = ProjectId(7),
            missing = ProjectId(9)
        );
        assert!(matches!(
            policy::check(&store, &user, &project),
            Err(Error::Denied)
        ));
        assert!(matches!(
            policy::check(&store, &user, &missing),
            Err(Error::Missing)
        ));
    }

    #[test]
    fn concurrent_permissions_allow_only_one_write_per_revision() {
        let backend = Store::new();
        name!(
            first_store = backend.clone(),
            first_user = UserId(1),
            first_project = ProjectId(7)
        );
        name!(
            second_store = backend.clone(),
            second_user = UserId(1),
            second_project = ProjectId(7)
        );
        let first = policy::check(&first_store, &first_user, &first_project)
            .unwrap()
            .bind(first_store, first_user, first_project);
        let second = policy::check(&second_store, &second_user, &second_project)
            .unwrap()
            .bind(second_store, second_user, second_project);
        let outcomes = std::thread::scope(|scope| {
            let first = scope.spawn(|| first.rename("first"));
            let second = scope.spawn(|| second.rename("second"));
            [first.join().unwrap(), second.join().unwrap()]
        });
        assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|r| **r == Err(Error::Conflict))
                .count(),
            1
        );
        assert!(matches!(
            backend.current_name().unwrap().as_str(),
            "first" | "second"
        ));
    }
}
