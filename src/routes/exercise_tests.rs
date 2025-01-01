use std::sync::Mutex;

use actix_web::{cookie::Cookie, test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::{
    middleware::session::SessionProtection,
    models::{exercise::{CreateExercise, Exercise, UpdateExercise}, session::Session, user::User},
    repositories::{auth_repository::{AuthError, AuthRepository}, exercise_repository::{ExerciseError, ExerciseRepository}}, routes::exercise::ExerciseResponse,
};

pub struct MockAuthRepo {
  sessions: Mutex<Vec<Session>>,
}

impl MockAuthRepo {
  pub fn new() -> Self {
      let mut sessions = vec![];
      sessions.push(Session {
          id: 1,
          user_id: 1,
          token: "user1-session".to_string(),
          csrf_token: "user1-csrf".to_string(),
          expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
          created_at: chrono::Utc::now().naive_utc(),
      });
      sessions.push(Session {
          id: 2,
          user_id: 2,
          token: "user2-session".to_string(),
          csrf_token: "user2-csrf".to_string(),
          expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
          created_at: chrono::Utc::now().naive_utc(),
      });
      Self {
          sessions: Mutex::new(sessions),
      }
  }
}

impl AuthRepository for MockAuthRepo {
  fn validate_session(&self, session_token: &str) -> Result<i64, AuthError> {
      let sessions = self.sessions.lock().unwrap();
      let session = sessions.iter()
          .find(|s| s.token == session_token)
          .ok_or(AuthError::InvalidSession)?;
      Ok(session.user_id)
  }

  // Implement other required methods with empty/mock implementations
  fn create_temp_session(&self, _csrf_token: String) -> Result<crate::models::temp_session::TempSession, AuthError> { unimplemented!() }
  fn create_session(&self, _user_id: i64, _session_id: String, _csrf_token: String) -> Result<Session, AuthError> { unimplemented!() }
  fn validate_csrf(&self, _session_id: &str, _csrf_token: &str) -> Result<(), AuthError> { unimplemented!() }
  fn verify_credentials(&self, _email: String, _password: String) -> Result<User, AuthError> { unimplemented!() }
  fn invalidate_session(&self, _session_token: &str) -> Result<(), AuthError> { unimplemented!() }
  fn create_user(&self, _email: String, _password: String) -> Result<User, AuthError> { unimplemented!() }
  fn delete_user(&self, _session_token: &str) -> Result<(), AuthError> { unimplemented!() }
}

pub struct MockExerciseRepo {
  exercises: Mutex<Vec<Exercise>>,
}

impl MockExerciseRepo {
  pub fn new() -> Self {
      Self {
          exercises: Mutex::new(vec![]),
      }
  }
}

impl ExerciseRepository for MockExerciseRepo {
  fn create_exercise(&self, user_id: i64, exercise: CreateExercise) -> Result<Exercise, ExerciseError> {
      let mut exercises = self.exercises.lock().unwrap();
      let new_exercise = Exercise::new(user_id, exercise.name, exercise.description);
      let exercise = Exercise {
          id: (exercises.len() + 1) as i64,
          uuid: new_exercise.uuid,
          user_id: new_exercise.user_id,
          name: new_exercise.name,
          description: new_exercise.description,
          created_at: new_exercise.created_at,
          updated_at: new_exercise.updated_at,
      };
      exercises.push(exercise.clone());
      Ok(exercise)
  }

  fn get_exercise(&self, user_id: i64, exercise_uuid: Uuid) -> Result<Exercise, ExerciseError> {
      let exercises = self.exercises.lock().unwrap();
      exercises.iter()
          .find(|e| e.uuid == exercise_uuid && e.user_id == user_id)
          .cloned()
          .ok_or(ExerciseError::NotFound)
  }

  fn list_exercises(&self, user_id: i64) -> Result<Vec<Exercise>, ExerciseError> {
      let exercises = self.exercises.lock().unwrap();
      Ok(exercises.iter().filter(|e| e.user_id == user_id).cloned().collect())
  }

  fn update_exercise(&self, user_id: i64, exercise_uuid: Uuid, exercise: UpdateExercise) -> Result<Exercise, ExerciseError> {
      let mut exercises = self.exercises.lock().unwrap();
      let exercise_index = exercises.iter()
          .position(|e| e.uuid == exercise_uuid && e.user_id == user_id)
          .ok_or(ExerciseError::NotFound)?;
      
      // Update the exercise fields
      exercises[exercise_index].name = exercise.name.unwrap_or(exercises[exercise_index].name.clone());
      if let Some(description) = exercise.description {
          exercises[exercise_index].description = Some(description);
      }
      exercises[exercise_index].updated_at = chrono::Utc::now().naive_utc();
      
      Ok(exercises[exercise_index].clone())
  }

  fn delete_exercise(&self, user_id: i64, exercise_uuid: Uuid) -> Result<(), ExerciseError> {
      let mut exercises = self.exercises.lock().unwrap();
      let initial_len = exercises.len();
      exercises.retain(|e| !(e.uuid == exercise_uuid && e.user_id == user_id));
      if exercises.len() < initial_len {
          Ok(())
      } else {
          Err(ExerciseError::NotFound)
      }
  }
}

#[actix_web::test]
async fn test_exercise_crud() {
    let auth_repo = web::Data::new(MockAuthRepo::new());
    let exercise_repo = web::Data::new(MockExerciseRepo::new());

    let app = test::init_service(
        App::new()
            .app_data(auth_repo.clone())
            .app_data(exercise_repo.clone())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .service(crate::routes::exercise::get_scope_exercise_id::<MockExerciseRepo>())
                    .service(crate::routes::exercise::get_scope::<MockExerciseRepo>())
            )
    ).await;

    // Create exercise
    let req = test::TestRequest::post()
        .uri("/exercises")
        .cookie(Cookie::new("session_id", "user1-session"))
        .set_json(json!({
            "name": "Push-ups",
            "description": "Basic bodyweight exercise"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercise: ExerciseResponse = test::read_body_json(resp).await;
    assert_eq!(exercise.name, "Push-ups");
    let exercise_uuid = exercise.uuid;

    // Get exercise
    let req = test::TestRequest::get()
        .uri(&format!("/exercises/{}", exercise_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercise: ExerciseResponse = test::read_body_json(resp).await;
    assert_eq!(exercise.name, "Push-ups");

    // List exercises
    let req = test::TestRequest::get()
        .uri("/exercises")
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercises: Vec<ExerciseResponse> = test::read_body_json(resp).await;
    assert_eq!(exercises.len(), 1);

    // Update exercise
    let req = test::TestRequest::put()
        .uri(&format!("/exercises/{}", exercise_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .set_json(json!({
            "name": "Diamond Push-ups",
            "description": "Advanced variation"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercise: ExerciseResponse = test::read_body_json(resp).await;
    assert_eq!(exercise.name, "Diamond Push-ups");

    // Delete exercise
    let req = test::TestRequest::delete()
        .uri(&format!("/exercises/{}", exercise_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify exercise was deleted
    let req = test::TestRequest::get()
        .uri(&format!("/exercises/{}", exercise_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[actix_web::test]
async fn test_exercise_isolation() {
    let auth_repo = web::Data::new(MockAuthRepo::new());
    let exercise_repo = web::Data::new(MockExerciseRepo::new());

    let app = test::init_service(
        App::new()
            .app_data(auth_repo.clone())
            .app_data(exercise_repo.clone())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .service(crate::routes::exercise::get_scope_exercise_id::<MockExerciseRepo>())
                    .service(crate::routes::exercise::get_scope::<MockExerciseRepo>())
            )
    ).await;

    // User 1 creates an exercise
    let req = test::TestRequest::post()
        .uri("/exercises")
        .cookie(Cookie::new("session_id", "user1-session"))
        .set_json(json!({
            "name": "User 1 Exercise",
            "description": "Test Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let user1_exercise: ExerciseResponse = test::read_body_json(resp).await;
    let user1_exercise_uuid = user1_exercise.uuid;

    // User 2 creates an exercise
    let req = test::TestRequest::post()
        .uri("/exercises")
        .cookie(Cookie::new("session_id", "user2-session"))
        .set_json(json!({
            "name": "User 2 Exercise",
            "description": "Test Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // User 1 should only see their exercise
    let req = test::TestRequest::get()
        .uri("/exercises")
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercises: Vec<ExerciseResponse> = test::read_body_json(resp).await;
    assert_eq!(exercises.len(), 1);
    assert_eq!(exercises[0].uuid, user1_exercise_uuid);

    // User 2 cannot access User 1's exercise
    let req = test::TestRequest::get()
        .uri(&format!("/exercises/{}", user1_exercise_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // User 2 cannot update User 1's exercise
    let req = test::TestRequest::put()
        .uri(&format!("/exercises/{}", user1_exercise_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .set_json(json!({
            "name": "Hacked Exercise",
            "description": "Hacked Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // User 2 cannot delete User 1's exercise
    let req = test::TestRequest::delete()
        .uri(&format!("/exercises/{}", user1_exercise_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
} 