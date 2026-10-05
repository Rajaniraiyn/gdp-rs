//! An async protected operation using bundled user, project, and evidence.
use ghostproof::{And, Named, name};

#[derive(Debug)]
struct UserId(u64);
#[derive(Debug)]
struct ProjectId(u64);

#[derive(Debug, PartialEq)]
enum Error {
    Denied,
    Missing,
    Backend,
}

struct Database;
impl Database {
    async fn role(&self, user: u64, project: u64) -> Result<bool, Error> {
        match project {
            0 => Err(Error::Missing),
            99 => Err(Error::Backend),
            _ => Ok(user == 1 && project == 7),
        }
    }
    async fn plan(&self, project: u64) -> Result<bool, Error> {
        Ok(project == 7)
    }
    // This raw write stays private to the application's protected boundary.
    async fn write_password(&self, project: u64, password: &str) -> Result<(), Error> {
        println!(
            "set password for project {project}: {} characters",
            password.len()
        );
        Ok(())
    }
}

mod policy {
    use super::*;

    #[ghostproof::proof]
    pub struct Admin<'u, 'p>;
    #[ghostproof::proof]
    pub struct Plan<'p>;
    #[ghostproof::proof]
    pub struct CanProtect<'u, 'p>;

    pub async fn admin<'u, 'p>(
        db: &Database,
        u: &Named<'u, UserId>,
        p: &Named<'p, ProjectId>,
    ) -> Result<Admin<'u, 'p>, Error> {
        if db.role(u.value().0, p.value().0).await? {
            Ok(Admin::issue(u, p))
        } else {
            Err(Error::Denied)
        }
    }
    pub async fn plan<'p>(db: &Database, p: &Named<'p, ProjectId>) -> Result<Plan<'p>, Error> {
        if db.plan(p.value().0).await? {
            Ok(Plan::issue(p))
        } else {
            Err(Error::Denied)
        }
    }
    // Explicit trusted inference: this policy requires both admin and plan.
    pub fn can_protect<'u, 'p>(
        u: &Named<'u, UserId>,
        p: &Named<'p, ProjectId>,
        facts: And<Admin<'u, 'p>, Plan<'p>>,
    ) -> CanProtect<'u, 'p> {
        let _ = facts.into_parts();
        CanProtect::issue(u, p)
    }
    impl<'u, 'p> CanProtectCapability<'u, 'p, UserId, ProjectId> {
        pub async fn set_password(&self, db: &Database, password: &str) -> Result<(), Error> {
            self.as_view().set_password(db, password).await
        }
    }
    impl<'view, 'u, 'p> CanProtectView<'view, 'u, 'p, UserId, ProjectId> {
        pub async fn set_password(&self, db: &Database, password: &str) -> Result<(), Error> {
            db.write_password(self.subject_1().value().0, password)
                .await
        }
    }
}

async fn handler(db: &Database, user_id: u64, project_id: u64) -> Result<(), Error> {
    name!(user = UserId(user_id), project = ProjectId(project_id));
    let (admin, plan) = futures::try_join!(
        policy::admin(db, &user, &project),
        policy::plan(db, &project)
    )?;
    let proof = policy::can_protect(&user, &project, And::new(admin, plan));
    let project = proof.bind(user, project);
    project.set_password(db, "secret").await
}

fn main() {
    let db = Database;
    futures::executor::block_on(async {
        assert_eq!(handler(&db, 1, 7).await, Ok(()));
        assert_eq!(handler(&db, 2, 7).await, Err(Error::Denied));
        assert_eq!(handler(&db, 1, 0).await, Err(Error::Missing));
        assert_eq!(handler(&db, 1, 99).await, Err(Error::Backend));
    });
}
