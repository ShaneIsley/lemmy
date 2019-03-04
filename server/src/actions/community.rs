extern crate diesel;
use schema::{community, community_user, community_follower};
use diesel::*;
use diesel::result::Error;
use {Crud, Followable, Joinable};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name="community"]
pub struct Community {
  pub id: i32,
  pub name: String,
  pub start_time: chrono::NaiveDateTime
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="community"]
pub struct CommunityForm<'a> {
    pub name: &'a str,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_user"]
pub struct CommunityUser {
  pub id: i32,
  pub community_id: i32,
  pub fedi_user_id: String,
  pub start_time: chrono::NaiveDateTime
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="community_user"]
pub struct CommunityUserForm<'a> {
  pub community_id: &'a i32,
  pub fedi_user_id: &'a str,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_follower"]
pub struct CommunityFollower {
  pub id: i32,
  pub community_id: i32,
  pub fedi_user_id: String,
  pub start_time: chrono::NaiveDateTime
}

#[derive(Insertable, AsChangeset, Clone, Copy)]
#[table_name="community_follower"]
pub struct CommunityFollowerForm<'a> {
  pub community_id: &'a i32,
  pub fedi_user_id: &'a str,
}


impl<'a> Crud<CommunityForm<'a>> for Community {
  fn read(conn: &PgConnection, community_id: i32) -> Community {
    use schema::community::dsl::*;
    community.find(community_id)
      .first::<Community>(conn)
      .expect("Error in query")
  }

  fn delete(conn: &PgConnection, community_id: i32) -> usize {
    use schema::community::dsl::*;
    diesel::delete(community.find(community_id))
      .execute(conn)
      .expect("Error deleting.")
  }

  fn create(conn: &PgConnection, new_community: CommunityForm) -> Result<Community, Error> {
    use schema::community::dsl::*;
      insert_into(community)
        .values(new_community)
        .get_result::<Community>(conn)
  }

  fn update(conn: &PgConnection, community_id: i32, new_community: CommunityForm) -> Community {
    use schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set(new_community)
      .get_result::<Community>(conn)
      .expect(&format!("Unable to find {}", community_id))
  }
}

impl<'a> Followable<CommunityFollowerForm<'a>> for CommunityFollower {
  fn follow(conn: &PgConnection, community_follower_form: CommunityFollowerForm) -> Result<CommunityFollower, Error> {
    use schema::community_follower::dsl::*;
    insert_into(community_follower)
      .values(community_follower_form)
      .get_result::<CommunityFollower>(conn)
  }
  fn ignore(conn: &PgConnection, community_follower_form: CommunityFollowerForm) -> usize {
    use schema::community_follower::dsl::*;
    diesel::delete(community_follower
      .filter(community_id.eq(community_follower_form.community_id))
      .filter(fedi_user_id.eq(community_follower_form.fedi_user_id)))
      .execute(conn)
      .expect("Error deleting.")
  }
}

impl<'a> Joinable<CommunityUserForm<'a>> for CommunityUser {
  fn join(conn: &PgConnection, community_user_form: CommunityUserForm) -> Result<CommunityUser, Error> {
    use schema::community_user::dsl::*;
    insert_into(community_user)
      .values(community_user_form)
      .get_result::<CommunityUser>(conn)
  }
  fn leave(conn: &PgConnection, community_user_form: CommunityUserForm) -> usize {
    use schema::community_user::dsl::*;
    diesel::delete(community_user
      .filter(community_id.eq(community_user_form.community_id))
      .filter(fedi_user_id.eq(community_user_form.fedi_user_id)))
      .execute(conn)
      .expect("Error deleting.")
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  use actions::user::*;
  use Crud;
 #[test]
  fn test_crud() {
    let conn = establish_connection();
    
    let new_community = CommunityForm {
      name: "TIL".into(),
    };

    let inserted_community = Community::create(&conn, new_community).unwrap();

    let expected_community = Community {
      id: inserted_community.id,
      name: "TIL".into(),
      start_time: inserted_community.start_time
    };

    let new_user = UserForm {
      name: "thom".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None
    };

    let inserted_user = User_::create(&conn, new_user).unwrap();

    let community_follower_form = CommunityFollowerForm {
      community_id: &inserted_community.id,
      fedi_user_id: "test".into()
    };

    let inserted_community_follower = CommunityFollower::follow(&conn, community_follower_form).unwrap();

    let expected_community_follower = CommunityFollower {
      id: inserted_community_follower.id,
      community_id: inserted_community.id,
      fedi_user_id: "test".into(),
      start_time: inserted_community_follower.start_time
    };
    
    let community_user_form = CommunityUserForm {
      community_id: &inserted_community.id,
      fedi_user_id: "test".into()
    };

    let inserted_community_user = CommunityUser::join(&conn, community_user_form).unwrap();

    let expected_community_user = CommunityUser {
      id: inserted_community_user.id,
      community_id: inserted_community.id,
      fedi_user_id: "test".into(),
      start_time: inserted_community_user.start_time
    };

    let read_community = Community::read(&conn, inserted_community.id);
    let updated_community = Community::update(&conn, inserted_community.id, new_community);
    let ignored_community = CommunityFollower::ignore(&conn, community_follower_form);
    let left_community = CommunityUser::leave(&conn, community_user_form);
    let num_deleted = Community::delete(&conn, inserted_community.id);
    User_::delete(&conn, inserted_user.id);

    assert_eq!(expected_community, read_community);
    assert_eq!(expected_community, inserted_community);
    assert_eq!(expected_community, updated_community);
    assert_eq!(expected_community_follower, inserted_community_follower);
    assert_eq!(expected_community_user, inserted_community_user);
    assert_eq!(1, ignored_community);
    assert_eq!(1, left_community);
    assert_eq!(1, num_deleted);

  }
}