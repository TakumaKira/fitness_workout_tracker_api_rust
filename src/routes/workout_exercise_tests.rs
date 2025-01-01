use actix_web::{cookie::Cookie, test, web, App};
use serde_json::json;
use std::sync::Mutex;
use uuid::Uuid;

use crate::{
    middleware::session::SessionProtection,
    models::{exercise::Exercise, session::Session, user::User, workout::Workout, workout_exercise::WorkoutExercise},
    repositories::{
        auth_repository::{AuthError, AuthRepository}, workout_exercise_repository::{WorkoutExerciseError, WorkoutExerciseRepository}
    }
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

pub struct MockWorkoutExerciseRepo {
    state: Mutex<(Vec<Workout>, Vec<Exercise>, Vec<WorkoutExercise>)>,
}

impl MockWorkoutExerciseRepo {
    pub fn new() -> Self {
        let workouts = vec![
            Workout {
                id: 1,
                uuid: Uuid::new_v4(),
                user_id: 1,
                name: "Test Workout for user 1".to_string(),
                description: None,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            Workout {
                id: 2,
                uuid: Uuid::new_v4(),
                user_id: 2,
                name: "Test Workout for user 2".to_string(),
                description: None,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
        ];
        let exercises = vec![
            Exercise {
                id: 1,
                uuid: Uuid::new_v4(),
                user_id: 1,
                name: "Test Exercise 1 for user 1".to_string(),
                description: None,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
            Exercise {
                id: 2,
                uuid: Uuid::new_v4(),
                user_id: 1,
                name: "Test Exercise 2 for user 1".to_string(),
                description: None,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            },
        ];
        Self {
            state: Mutex::new((workouts, exercises, vec![])),
        }
    }
}

impl WorkoutExerciseRepository for MockWorkoutExerciseRepo {
    fn add_exercise_to_workout(&self, user_id: i64, workout_uuid: Uuid, exercise_uuid: Uuid, order: i32) -> Result<(), WorkoutExerciseError> {
        let mut state = self.state.lock().unwrap();
        let (workouts, exercises, workout_exercises) = &mut *state;

        let workout_id = workouts
            .iter()
            .find(|w| w.uuid == workout_uuid && w.user_id == user_id)
            .ok_or(WorkoutExerciseError::WorkoutNotFound)?
            .id;

        let exercise_id = exercises
            .iter()
            .find(|e| e.uuid == exercise_uuid && e.user_id == user_id)
            .ok_or(WorkoutExerciseError::ExerciseNotFound)?
            .id;

        workout_exercises.push(WorkoutExercise {
            workout_id,
            exercise_id,
            user_id,
            order,
        });
        Ok(())
    }

    fn remove_exercise_from_workout(&self, user_id: i64, workout_uuid: Uuid, exercise_uuid: Uuid) -> Result<(), WorkoutExerciseError> {
        let mut state = self.state.lock().unwrap();
        let (workouts, exercises, workout_exercises) = &mut *state;

        let workout_id = workouts
            .iter()
            .find(|w| w.uuid == workout_uuid && w.user_id == user_id)
            .ok_or(WorkoutExerciseError::WorkoutNotFound)?
            .id;

        let exercise_id = exercises
            .iter()
            .find(|e| e.uuid == exercise_uuid && e.user_id == user_id)
            .ok_or(WorkoutExerciseError::ExerciseNotFound)?
            .id;

        let initial_len = workout_exercises.len();
        workout_exercises.retain(|we| 
            !(we.workout_id == workout_id && 
              we.exercise_id == exercise_id && 
              we.user_id == user_id)
        );
        
        if workout_exercises.len() < initial_len {
            Ok(())
        } else {
            Err(WorkoutExerciseError::NotFound)
        }
    }

    fn list_workout_exercises(&self, user_id: i64, workout_uuid: Uuid) -> Result<Vec<(Exercise, i32)>, WorkoutExerciseError> {
        let state = self.state.lock().unwrap();
        let (workouts, exercises, workout_exercises) = &*state;

        let workout_id = workouts
            .iter()
            .find(|w| w.uuid == workout_uuid && w.user_id == user_id)
            .ok_or(WorkoutExerciseError::WorkoutNotFound)?
            .id;

        let result: Vec<(Exercise, i32)> = workout_exercises
            .iter()
            .filter(|we| we.workout_id == workout_id && we.user_id == user_id)
            .filter_map(|we| {
                exercises
                    .iter()
                    .find(|e| e.id == we.exercise_id)
                    .map(|e| (e.clone(), we.order))
            })
            .collect();

        Ok(result)
    }
}

#[actix_web::test]
async fn test_workout_exercises() {
    let auth_repo = web::Data::new(MockAuthRepo::new());
    let workout_exercise_repo = web::Data::new(MockWorkoutExerciseRepo::new());

    // Get UUIDs before initializing the service
    let workout_uuid;
    let exercise_uuid;
    {  // Explicit scope to ensure lock is released
        let state = workout_exercise_repo.state.lock().unwrap();
        let (workouts, exercises, _) = &*state;
        workout_uuid = workouts.iter().find(|w| w.user_id == 1).unwrap().uuid;
        exercise_uuid = exercises.iter().find(|e| e.user_id == 1).unwrap().uuid;
    }  // Lock is released here

    let app = test::init_service(
        App::new()
            .app_data(auth_repo.clone())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .app_data(workout_exercise_repo.clone())
                    .service(crate::routes::workout_exercise::get_scope_workout_id_exercises_exercise_id::<MockWorkoutExerciseRepo>())
                    .service(crate::routes::workout_exercise::get_scope_workout_id_exercises::<MockWorkoutExerciseRepo>())
            )
    ).await;

    // Add exercise to workout
    let req = test::TestRequest::post()
        .uri(&format!("/workouts/{}/exercises", workout_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .set_json(json!({
            "exercise_uuid": exercise_uuid,
            "order": 1
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // List workout exercises
    let req = test::TestRequest::get()
        .uri(&format!("/workouts/{}/exercises", workout_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercises: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(exercises.len(), 1);

    // Remove exercise from workout
    let req = test::TestRequest::delete()
        .uri(&format!("/workouts/{}/exercises/{}", workout_uuid, exercise_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify exercise was removed
    let req = test::TestRequest::get()
        .uri(&format!("/workouts/{}/exercises", workout_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let exercises: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(exercises.len(), 0);
}

#[actix_web::test]
async fn test_workout_exercise_isolation() {
    let auth_repo = web::Data::new(MockAuthRepo::new());
    let workout_exercise_repo = web::Data::new(MockWorkoutExerciseRepo::new());

    // Get all needed data before initializing the service
    let (user1_workout_uuid, user1_exercise_uuid);
    {
        let state = workout_exercise_repo.state.lock().unwrap();
        let (workouts, exercises, _) = &*state;
        
        // Clone or copy all needed data
        user1_workout_uuid = workouts.iter()
            .find(|w| w.user_id == 1)
            .map(|w| w.uuid)
            .unwrap();
        user1_exercise_uuid = exercises.iter()
            .find(|e| e.user_id == 1)
            .map(|e| e.uuid)
            .unwrap();
    }  // Lock is released here

    let app = test::init_service(
        App::new()
            .app_data(auth_repo.clone())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .app_data(workout_exercise_repo.clone())
                    .service(crate::routes::workout_exercise::get_scope_workout_id_exercises_exercise_id::<MockWorkoutExerciseRepo>())
                    .service(crate::routes::workout_exercise::get_scope_workout_id_exercises::<MockWorkoutExerciseRepo>())
            )
    ).await;

    // User 1 adds exercise to their workout
    let req = test::TestRequest::post()
        .uri(&format!("/workouts/{}/exercises", user1_workout_uuid))
        .cookie(Cookie::new("session_id", "user1-session"))
        .set_json(json!({
            "exercise_uuid": user1_exercise_uuid,
            "order": 1
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // User 2 tries to access User 1's workout exercises (should get 404)
    let req = test::TestRequest::get()
        .uri(&format!("/workouts/{}/exercises", user1_workout_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // User 2 tries to add exercise to User 1's workout (should fail)
    let req = test::TestRequest::post()
        .uri(&format!("/workouts/{}/exercises", user1_workout_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .set_json(json!({
            "exercise_uuid": user1_exercise_uuid,
            "order": 1
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // User 2 tries to remove exercise from User 1's workout (should fail)
    let req = test::TestRequest::delete()
        .uri(&format!("/workouts/{}/exercises/{}", user1_workout_uuid, user1_exercise_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
} 