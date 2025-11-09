//! Tests for authentication pallet

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_session_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;

        // Register user first
        register_user(account);

        // Set timestamp
        Timestamp::set_timestamp(1000);

        // Create session
        assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));

        // Get session ID from event
        let events = System::events();
        let session_id = if let RuntimeEvent::Authentication(Event::SessionCreated {
            session_id,
            ..
        }) = events.last().unwrap().event
        {
            session_id
        } else {
            panic!("Expected SessionCreated event");
        };

        // Verify session was created
        let session = Authentication::sessions(session_id).unwrap();
        assert_eq!(session.account, account);
        assert_eq!(session.created_at, 1000);
        assert_eq!(session.expires_at, 1000 + 86400000); // 24 hours later
        assert!(session.active);

        // Verify session is in account's session list
        assert!(Authentication::account_sessions(account).contains(&session_id));
    });
}

#[test]
fn create_session_fails_without_identity() {
    new_test_ext().execute_with(|| {
        let account = 1u64;

        assert_noop!(
            Authentication::create_session(RuntimeOrigin::signed(account)),
            Error::<Test>::InvalidIdentity
        );
    });
}

#[test]
fn create_multiple_sessions_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // Create multiple sessions
        for _ in 0..5 {
            assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));
        }

        // Verify all sessions exist
        let sessions = Authentication::account_sessions(account);
        assert_eq!(sessions.len(), 5);
    });
}

#[test]
fn revoke_session_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // Create session
        assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));

        let events = System::events();
        let session_id = if let RuntimeEvent::Authentication(Event::SessionCreated {
            session_id,
            ..
        }) = events.last().unwrap().event
        {
            session_id
        } else {
            panic!("Expected SessionCreated event");
        };

        // Revoke session
        assert_ok!(Authentication::revoke_session(
            RuntimeOrigin::signed(account),
            session_id
        ));

        // Verify session is inactive
        let session = Authentication::sessions(session_id).unwrap();
        assert!(!session.active);

        // Check event
        System::assert_last_event(Event::SessionRevoked { session_id }.into());
    });
}

#[test]
fn revoke_session_fails_when_not_owner() {
    new_test_ext().execute_with(|| {
        let account1 = 1u64;
        let account2 = 2u64;

        register_user(account1);
        register_user(account2);

        // Create session for account1
        assert_ok!(Authentication::create_session(RuntimeOrigin::signed(
            account1
        )));

        let events = System::events();
        let session_id = if let RuntimeEvent::Authentication(Event::SessionCreated {
            session_id,
            ..
        }) = events.last().unwrap().event
        {
            session_id
        } else {
            panic!("Expected SessionCreated event");
        };

        // Try to revoke as account2
        assert_noop!(
            Authentication::revoke_session(RuntimeOrigin::signed(account2), session_id),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn revoke_session_fails_when_not_found() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        let fake_session_id = sp_core::H256::from_low_u64_be(999);

        assert_noop!(
            Authentication::revoke_session(RuntimeOrigin::signed(account), fake_session_id),
            Error::<Test>::SessionNotFound
        );
    });
}

#[test]
fn create_api_key_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        let key_hash = create_key_hash("my_secret_key");
        let name = sp_runtime::BoundedVec::try_from(b"Production API".to_vec()).unwrap();

        // Set timestamp
        Timestamp::set_timestamp(5000);

        // Create API key
        assert_ok!(Authentication::create_api_key(
            RuntimeOrigin::signed(account),
            key_hash,
            name.clone()
        ));

        // Verify API key was created
        let api_key = Authentication::api_keys(key_hash).unwrap();
        assert_eq!(api_key.account, account);
        assert_eq!(api_key.name, name);
        assert_eq!(api_key.created_at, 5000);
        assert!(api_key.last_used.is_none());
        assert!(api_key.active);

        // Check event
        System::assert_last_event(
            Event::ApiKeyCreated {
                account,
                key_hash,
            }
            .into(),
        );
    });
}

#[test]
fn create_api_key_fails_without_identity() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let key_hash = create_key_hash("my_secret_key");
        let name = sp_runtime::BoundedVec::try_from(b"Test".to_vec()).unwrap();

        assert_noop!(
            Authentication::create_api_key(RuntimeOrigin::signed(account), key_hash, name),
            Error::<Test>::InvalidIdentity
        );
    });
}

#[test]
fn create_multiple_api_keys_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // Create multiple API keys
        for i in 0..3 {
            let key_hash = create_key_hash(&format!("key_{}", i));
            let name = sp_runtime::BoundedVec::try_from(format!("Key {}", i).as_bytes().to_vec())
                .unwrap();

            assert_ok!(Authentication::create_api_key(
                RuntimeOrigin::signed(account),
                key_hash,
                name
            ));
        }
    });
}

#[test]
fn revoke_api_key_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        let key_hash = create_key_hash("my_secret_key");
        let name = sp_runtime::BoundedVec::try_from(b"Test Key".to_vec()).unwrap();

        // Create API key
        assert_ok!(Authentication::create_api_key(
            RuntimeOrigin::signed(account),
            key_hash,
            name
        ));

        // Revoke API key
        assert_ok!(Authentication::revoke_api_key(
            RuntimeOrigin::signed(account),
            key_hash
        ));

        // Verify API key is inactive
        let api_key = Authentication::api_keys(key_hash).unwrap();
        assert!(!api_key.active);

        // Check event
        System::assert_last_event(Event::ApiKeyRevoked { key_hash }.into());
    });
}

#[test]
fn revoke_api_key_fails_when_not_owner() {
    new_test_ext().execute_with(|| {
        let account1 = 1u64;
        let account2 = 2u64;

        register_user(account1);
        register_user(account2);

        let key_hash = create_key_hash("my_secret_key");
        let name = sp_runtime::BoundedVec::try_from(b"Test Key".to_vec()).unwrap();

        // Create API key for account1
        assert_ok!(Authentication::create_api_key(
            RuntimeOrigin::signed(account1),
            key_hash,
            name
        ));

        // Try to revoke as account2
        assert_noop!(
            Authentication::revoke_api_key(RuntimeOrigin::signed(account2), key_hash),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn revoke_api_key_fails_when_not_found() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        let fake_key_hash = create_key_hash("nonexistent_key");

        assert_noop!(
            Authentication::revoke_api_key(RuntimeOrigin::signed(account), fake_key_hash),
            Error::<Test>::ApiKeyNotFound
        );
    });
}

#[test]
fn session_validation_helper_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // Set timestamp
        Timestamp::set_timestamp(1000);

        // Create session
        assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));

        let events = System::events();
        let session_id = if let RuntimeEvent::Authentication(Event::SessionCreated {
            session_id,
            ..
        }) = events.last().unwrap().event
        {
            session_id
        } else {
            panic!("Expected SessionCreated event");
        };

        // Check session is valid at current time
        assert!(Authentication::is_session_valid(&session_id, 1000));

        // Check session is valid before expiry
        assert!(Authentication::is_session_valid(
            &session_id,
            1000 + 86400000 - 1
        ));

        // Check session is invalid after expiry
        assert!(!Authentication::is_session_valid(
            &session_id,
            1000 + 86400000 + 1
        ));

        // Revoke session
        assert_ok!(Authentication::revoke_session(
            RuntimeOrigin::signed(account),
            session_id
        ));

        // Check session is now invalid even at valid time
        assert!(!Authentication::is_session_valid(&session_id, 1000));
    });
}

#[test]
fn session_expires_correctly() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // Create session at t=1000
        Timestamp::set_timestamp(1000);
        assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));

        let events = System::events();
        let session_id = if let RuntimeEvent::Authentication(Event::SessionCreated {
            session_id,
            ..
        }) = events.last().unwrap().event
        {
            session_id
        } else {
            panic!("Expected SessionCreated event");
        };

        // Session should be valid immediately
        assert!(Authentication::is_session_valid(&session_id, 1000));

        // Session should be valid just before expiry (86400000ms = 24 hours)
        assert!(Authentication::is_session_valid(
            &session_id,
            1000 + 86400000 - 1
        ));

        // Session should be invalid right at expiry
        assert!(!Authentication::is_session_valid(
            &session_id,
            1000 + 86400000
        ));

        // Session should be invalid after expiry
        assert!(!Authentication::is_session_valid(
            &session_id,
            1000 + 86400000 + 1000
        ));
    });
}

#[test]
fn max_sessions_can_be_reached() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // Create maximum number of sessions (10)
        for _ in 0..10 {
            assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));
        }

        // Try to create one more - should fail
        assert_noop!(
            Authentication::create_session(RuntimeOrigin::signed(account)),
            Error::<Test>::MaxSessionsReached
        );
    });
}

#[test]
fn api_key_name_variations_work() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        let test_names = vec![
            "A",
            "Short",
            "Production API Key",
            "Development-Test-Environment-Key",
            "API_KEY_WITH_UNDERSCORES",
        ];

        for (i, name_str) in test_names.iter().enumerate() {
            let key_hash = create_key_hash(&format!("key_{}", i));
            let name = sp_runtime::BoundedVec::try_from(name_str.as_bytes().to_vec()).unwrap();

            assert_ok!(Authentication::create_api_key(
                RuntimeOrigin::signed(account),
                key_hash,
                name
            ));
        }
    });
}

#[test]
fn session_lifecycle_complete_test() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // 1. Create session
        Timestamp::set_timestamp(5000);
        assert_ok!(Authentication::create_session(RuntimeOrigin::signed(account)));

        let events = System::events();
        let session_id = if let RuntimeEvent::Authentication(Event::SessionCreated {
            session_id,
            ..
        }) = events.last().unwrap().event
        {
            session_id
        } else {
            panic!("Expected SessionCreated event");
        };

        // 2. Verify session details
        let session = Authentication::sessions(session_id).unwrap();
        assert_eq!(session.account, account);
        assert_eq!(session.created_at, 5000);
        assert!(session.active);

        // 3. Use session (validate)
        assert!(Authentication::is_session_valid(&session_id, 5000));
        assert!(Authentication::is_session_valid(&session_id, 50000));

        // 4. Revoke session
        assert_ok!(Authentication::revoke_session(
            RuntimeOrigin::signed(account),
            session_id
        ));

        // 5. Verify session is revoked
        let session = Authentication::sessions(session_id).unwrap();
        assert!(!session.active);
        assert!(!Authentication::is_session_valid(&session_id, 5000));
    });
}

#[test]
fn api_key_lifecycle_complete_test() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        register_user(account);

        // 1. Create API key
        let key_hash = create_key_hash("production_key");
        let name = sp_runtime::BoundedVec::try_from(b"Production".to_vec()).unwrap();

        Timestamp::set_timestamp(10000);
        assert_ok!(Authentication::create_api_key(
            RuntimeOrigin::signed(account),
            key_hash,
            name.clone()
        ));

        // 2. Verify API key details
        let api_key = Authentication::api_keys(key_hash).unwrap();
        assert_eq!(api_key.account, account);
        assert_eq!(api_key.name, name);
        assert_eq!(api_key.created_at, 10000);
        assert!(api_key.active);
        assert!(api_key.last_used.is_none());

        // 3. Revoke API key
        assert_ok!(Authentication::revoke_api_key(
            RuntimeOrigin::signed(account),
            key_hash
        ));

        // 4. Verify API key is revoked
        let api_key = Authentication::api_keys(key_hash).unwrap();
        assert!(!api_key.active);
    });
}
