use actix_web::{cookie::Cookie, test, web, App};
use serde_json::json;
use uuid::Uuid;
use std::sync::Mutex;

use crate::{
    middleware::session::SessionProtection,
    models::{session::Session, user::User, workout::Workout},
    repositories::{
        auth_repository::{AuthError, AuthRepository},
        workout_repository::{WorkoutError, WorkoutRepository},
    },
};

pub struct MockWorkoutRepo {
    workouts: Mutex<Vec<Workout>>,
}

impl MockWorkoutRepo {
    pub fn new() -> Self {
        Self {
            workouts: Mutex::new(vec![]),
        }
    }
}

impl WorkoutRepository for MockWorkoutRepo {
    fn create_workout(&self, user_id: i64, workout: crate::models::workout::CreateWorkout) -> Result<Workout, WorkoutError> {
        let mut workouts = self.workouts.lock().unwrap();
        let new_workout = Workout::new(user_id, workout.name, workout.description);
        let workout = Workout {
            id: (workouts.len() + 1) as i64,
            uuid: new_workout.uuid,
            user_id,
            name: new_workout.name,
            description: new_workout.description,
            created_at: new_workout.created_at,
            updated_at: new_workout.updated_at,
        };
        workouts.push(workout.clone());
        Ok(workout)
    }

    fn get_workout(&self, user_id: i64, workout_uuid: Uuid) -> Result<Workout, WorkoutError> {
        let workouts = self.workouts.lock().unwrap();
        let workout = workouts.iter()
            .find(|w| w.uuid == workout_uuid)
            .ok_or(WorkoutError::NotFound)?;
        if workout.user_id != user_id {
            return Err(WorkoutError::Unauthorized);
        }
        Ok(workout.clone())
    }

    fn list_workouts(&self, user_id: i64) -> Result<Vec<Workout>, WorkoutError> {
        let workouts = self.workouts.lock().unwrap();
        Ok(workouts.iter()
            .filter(|w| w.user_id == user_id)
            .cloned()
            .collect())
    }

    fn update_workout(&self, user_id: i64, workout_uuid: Uuid, update: crate::models::workout::UpdateWorkout) -> Result<Workout, WorkoutError> {
        let mut workouts = self.workouts.lock().unwrap();
        let workout = workouts.iter_mut()
            .find(|w| w.uuid == workout_uuid)
            .ok_or(WorkoutError::NotFound)?;
        
        if workout.user_id != user_id {
            return Err(WorkoutError::Unauthorized);
        }

        if let Some(name) = update.name {
            workout.name = name;
        }
        workout.description = update.description;
        workout.updated_at = chrono::Utc::now().naive_utc();
        
        Ok(workout.clone())
    }

    fn delete_workout(&self, user_id: i64, workout_uuid: Uuid) -> Result<(), WorkoutError> {
        let mut workouts = self.workouts.lock().unwrap();
        let initial_len = workouts.len();
        workouts.retain(|w| !(w.uuid == workout_uuid && w.user_id == user_id));
        
        if workouts.len() < initial_len {
            Ok(())
        } else {
            Err(WorkoutError::NotFound)
        }
    }
}

pub struct MockAuthRepo {
    sessions: Mutex<Vec<Session>>,
}

impl MockAuthRepo {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(vec![]),
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
    fn create_user(&self, _email: String, _password: String) -> Result<User, AuthError> { unimplemented!() }
    fn invalidate_session(&self, _session_token: &str) -> Result<(), AuthError> { unimplemented!() }
    fn delete_user(&self, _session_token: &str) -> Result<(), AuthError> { unimplemented!() }
}

#[actix_web::test]
async fn test_workout_crud() {
    let auth_repo = web::Data::new(MockAuthRepo::new());
    let workout_repo = web::Data::new(MockWorkoutRepo::new());
    
    // Setup test session
    auth_repo.sessions.lock().unwrap().push(Session {
        id: 1,
        user_id: 1,
        token: "test-session".to_string(),
        csrf_token: "test-csrf".to_string(),
        expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
        created_at: chrono::Utc::now().naive_utc(),
    });

    let app = test::init_service(
        App::new()
            .app_data(auth_repo.clone())
            .app_data(workout_repo.clone())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .service(crate::routes::workout::get_scope_with_resource::<MockWorkoutRepo>())
                    .service(crate::routes::workout::get_scope::<MockWorkoutRepo>())
            )
    ).await;

    // Test Create
    let req = test::TestRequest::post()
        .uri("/workouts")
        .cookie(Cookie::new("session_id", "test-session"))
        .set_json(json!({
            "name": "Test Workout",
            "description": "Test Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let workout: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(workout["name"], "Test Workout");
    let workout_uuid = workout["uuid"].as_str().unwrap();
    assert!(workout_uuid.len() > 0);

    // Test List
    let req = test::TestRequest::get()
        .uri("/workouts")
        .cookie(Cookie::new("session_id", "test-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let workouts: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(workouts.len(), 1);
    assert_eq!(workouts[0]["uuid"], workout_uuid);

    // Test Get
    let req = test::TestRequest::get()
        .uri(&format!("/workouts/{}", workout_uuid))
        .cookie(Cookie::new("session_id", "test-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test Update
    let req = test::TestRequest::put()
        .uri(&format!("/workouts/{}", workout_uuid))
        .cookie(Cookie::new("session_id", "test-session"))
        .set_json(json!({
            "name": "Updated Workout",
            "description": "Updated Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let updated: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(updated["name"], "Updated Workout");

    // Test Delete
    let req = test::TestRequest::delete()
        .uri(&format!("/workouts/{}", workout_uuid))
        .cookie(Cookie::new("session_id", "test-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    // Verify deletion
    let req = test::TestRequest::get()
        .uri(&format!("/workouts/{}", workout_uuid))
        .cookie(Cookie::new("session_id", "test-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_workout_isolation() {
    let auth_repo = web::Data::new(MockAuthRepo::new());
    let workout_repo = web::Data::new(MockWorkoutRepo::new());
    
    // Setup two test sessions for different users
    auth_repo.sessions.lock().unwrap().extend(vec![
        Session {
            id: 1,
            user_id: 1,
            token: "user1-session".to_string(),
            csrf_token: "test-csrf".to_string(),
            expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
            created_at: chrono::Utc::now().naive_utc(),
        },
        Session {
            id: 2,
            user_id: 2,
            token: "user2-session".to_string(),
            csrf_token: "test-csrf".to_string(),
            expires_at: chrono::Utc::now().naive_utc() + chrono::Duration::hours(1),
            created_at: chrono::Utc::now().naive_utc(),
        },
    ]);

    let app = test::init_service(
        App::new()
            .app_data(auth_repo.clone())
            .app_data(workout_repo.clone())
            .service(
                web::scope("")
                    .wrap(SessionProtection::<MockAuthRepo>::new())
                    .service(crate::routes::workout::get_scope::<MockWorkoutRepo>())
                    .service(crate::routes::workout::get_scope_with_resource::<MockWorkoutRepo>())
            )
    ).await;

    // User 1 creates a workout
    let req = test::TestRequest::post()
        .uri("/workouts")
        .cookie(Cookie::new("session_id", "user1-session"))
        .set_json(json!({
            "name": "User 1 Workout",
            "description": "Test Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let user1_workout: serde_json::Value = test::read_body_json(resp).await;
    let user1_workout_uuid = user1_workout["uuid"].as_str().unwrap();

    // User 2 creates a workout
    let req = test::TestRequest::post()
        .uri("/workouts")
        .cookie(Cookie::new("session_id", "user2-session"))
        .set_json(json!({
            "name": "User 2 Workout",
            "description": "Test Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let user2_workout: serde_json::Value = test::read_body_json(resp).await;
    let user2_workout_uuid = user2_workout["uuid"].as_str().unwrap();

    // User 1 should only see their workout
    let req = test::TestRequest::get()
        .uri("/workouts")
        .cookie(Cookie::new("session_id", "user1-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let workouts: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(workouts.len(), 1);
    assert_eq!(workouts[0]["uuid"], user1_workout_uuid);

    // User 2 should only see their workout
    let req = test::TestRequest::get()
        .uri("/workouts")
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let workouts: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(workouts.len(), 1);
    assert_eq!(workouts[0]["uuid"], user2_workout_uuid);

    // User 2 cannot access User 1's workout
    let req = test::TestRequest::get()
        .uri(&format!("/workouts/{}", user1_workout_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // User 2 cannot update User 1's workout
    let req = test::TestRequest::put()
        .uri(&format!("/workouts/{}", user1_workout_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .set_json(json!({
            "name": "Hacked Workout",
            "description": "Hacked Description"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // User 2 cannot delete User 1's workout
    let req = test::TestRequest::delete()
        .uri(&format!("/workouts/{}", user1_workout_uuid))
        .cookie(Cookie::new("session_id", "user2-session"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
} 