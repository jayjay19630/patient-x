//! Tests for identity-registry pallet

use crate::{mock::*, Error, Event, UserRole, VerificationStatus};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_identity_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice Patient");
        let email_hash = create_email_hash("alice@example.com");

        // Register identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did.clone(),
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        // Check identity was stored
        let identity = IdentityRegistry::identities(account).unwrap();
        assert_eq!(identity.did, did);
        assert_eq!(identity.role, UserRole::Patient);
        assert_eq!(identity.name, name);
        assert_eq!(identity.email_hash, email_hash);
        assert_eq!(identity.verification_status, VerificationStatus::Unverified);
        assert!(identity.active);

        // Check DID mapping
        assert_eq!(IdentityRegistry::did_to_account(&did), Some(account));

        // Check event was emitted
        System::assert_last_event(
            Event::IdentityRegistered {
                account,
                did,
                role: UserRole::Patient,
            }
            .into(),
        );
    });
}

#[test]
fn register_identity_fails_with_duplicate_account() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did1 = create_did("patient1");
        let did2 = create_did("patient2");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register first identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did1,
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        // Try to register second identity for same account
        assert_noop!(
            IdentityRegistry::register_identity(
                RuntimeOrigin::signed(account),
                did2,
                UserRole::Patient,
                name,
                email_hash,
            ),
            Error::<Test>::IdentityAlreadyExists
        );
    });
}

#[test]
fn register_identity_fails_with_duplicate_did() {
    new_test_ext().execute_with(|| {
        let account1 = 1u64;
        let account2 = 2u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register first identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account1),
            did.clone(),
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        // Try to register with same DID
        assert_noop!(
            IdentityRegistry::register_identity(
                RuntimeOrigin::signed(account2),
                did,
                UserRole::Patient,
                name,
                email_hash,
            ),
            Error::<Test>::DIDAlreadyExists
        );
    });
}

#[test]
fn register_identity_fails_with_invalid_did() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let short_did = sp_runtime::BoundedVec::try_from(b"short".to_vec()).unwrap();
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        assert_noop!(
            IdentityRegistry::register_identity(
                RuntimeOrigin::signed(account),
                short_did,
                UserRole::Patient,
                name,
                email_hash,
            ),
            Error::<Test>::InvalidDID
        );
    });
}

#[test]
fn register_identity_fails_with_empty_name() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let empty_name = sp_runtime::BoundedVec::try_from(b"".to_vec()).unwrap();
        let email_hash = create_email_hash("alice@example.com");

        assert_noop!(
            IdentityRegistry::register_identity(
                RuntimeOrigin::signed(account),
                did,
                UserRole::Patient,
                empty_name,
                email_hash,
            ),
            Error::<Test>::InvalidName
        );
    });
}

#[test]
fn update_identity_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        // Update name
        let new_name = create_name("Alice Updated");
        assert_ok!(IdentityRegistry::update_identity(
            RuntimeOrigin::signed(account),
            Some(new_name.clone()),
            None,
        ));

        let identity = IdentityRegistry::identities(account).unwrap();
        assert_eq!(identity.name, new_name);

        // Update email hash
        let new_email_hash = create_email_hash("alice.new@example.com");
        assert_ok!(IdentityRegistry::update_identity(
            RuntimeOrigin::signed(account),
            None,
            Some(new_email_hash),
        ));

        let identity = IdentityRegistry::identities(account).unwrap();
        assert_eq!(identity.email_hash, new_email_hash);

        // Check event was emitted
        System::assert_last_event(Event::IdentityUpdated { account }.into());
    });
}

#[test]
fn update_identity_fails_when_not_registered() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let new_name = create_name("Alice");

        assert_noop!(
            IdentityRegistry::update_identity(
                RuntimeOrigin::signed(account),
                Some(new_name),
                None,
            ),
            Error::<Test>::IdentityNotFound
        );
    });
}

#[test]
fn update_identity_fails_when_deactivated() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register and deactivate identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        assert_ok!(IdentityRegistry::deactivate_identity(RuntimeOrigin::signed(
            account
        )));

        // Try to update
        let new_name = create_name("Alice Updated");
        assert_noop!(
            IdentityRegistry::update_identity(
                RuntimeOrigin::signed(account),
                Some(new_name),
                None,
            ),
            Error::<Test>::IdentityNotActive
        );
    });
}

#[test]
fn request_verification_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        // Request verification
        assert_ok!(IdentityRegistry::request_verification(RuntimeOrigin::signed(
            account
        )));

        // Check status changed
        let identity = IdentityRegistry::identities(account).unwrap();
        assert_eq!(identity.verification_status, VerificationStatus::Pending);

        // Check queue entry
        assert!(IdentityRegistry::verification_queue(account).is_some());

        // Check event was emitted
        System::assert_last_event(Event::VerificationRequested { account }.into());
    });
}

#[test]
fn request_verification_fails_when_already_pending() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register and request verification
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        assert_ok!(IdentityRegistry::request_verification(RuntimeOrigin::signed(
            account
        )));

        // Try to request again
        assert_noop!(
            IdentityRegistry::request_verification(RuntimeOrigin::signed(account)),
            Error::<Test>::VerificationAlreadyPending
        );
    });
}

#[test]
fn verify_identity_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let auditor = 2u64;
        let patient_did = create_did("patient1");
        let auditor_did = create_did("auditor1");
        let name = create_name("Patient");
        let auditor_name = create_name("Auditor");
        let email_hash = create_email_hash("test@example.com");

        // Register patient
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(patient),
            patient_did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        // Register auditor
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(auditor),
            auditor_did,
            UserRole::Auditor,
            auditor_name,
            email_hash,
        ));

        // Request verification
        assert_ok!(IdentityRegistry::request_verification(RuntimeOrigin::signed(
            patient
        )));

        // Verify identity
        assert_ok!(IdentityRegistry::verify_identity(
            RuntimeOrigin::signed(auditor),
            patient,
        ));

        // Check status changed
        let identity = IdentityRegistry::identities(patient).unwrap();
        assert_eq!(identity.verification_status, VerificationStatus::Verified);

        // Check removed from queue
        assert!(IdentityRegistry::verification_queue(patient).is_none());

        // Check event was emitted
        System::assert_last_event(
            Event::IdentityVerified {
                account: patient,
                verifier: auditor,
            }
            .into(),
        );
    });
}

#[test]
fn verify_identity_fails_when_not_auditor() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;
        let patient_did = create_did("patient1");
        let researcher_did = create_did("researcher1");
        let name = create_name("User");
        let email_hash = create_email_hash("test@example.com");

        // Register both users
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(patient),
            patient_did,
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(researcher),
            researcher_did,
            UserRole::Researcher,
            name,
            email_hash,
        ));

        assert_ok!(IdentityRegistry::request_verification(RuntimeOrigin::signed(
            patient
        )));

        // Try to verify as non-auditor
        assert_noop!(
            IdentityRegistry::verify_identity(RuntimeOrigin::signed(researcher), patient),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn reject_verification_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let auditor = 2u64;
        let patient_did = create_did("patient1");
        let auditor_did = create_did("auditor1");
        let name = create_name("User");
        let email_hash = create_email_hash("test@example.com");

        // Setup
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(patient),
            patient_did,
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(auditor),
            auditor_did,
            UserRole::Auditor,
            name,
            email_hash,
        ));

        assert_ok!(IdentityRegistry::request_verification(RuntimeOrigin::signed(
            patient
        )));

        // Reject verification
        let reason = sp_runtime::BoundedVec::try_from(b"Invalid documents".to_vec()).unwrap();
        assert_ok!(IdentityRegistry::reject_verification(
            RuntimeOrigin::signed(auditor),
            patient,
            reason.clone(),
        ));

        // Check status
        let identity = IdentityRegistry::identities(patient).unwrap();
        assert_eq!(identity.verification_status, VerificationStatus::Rejected);

        // Check removed from queue
        assert!(IdentityRegistry::verification_queue(patient).is_none());

        // Check event
        System::assert_last_event(
            Event::VerificationRejected {
                account: patient,
                reason,
            }
            .into(),
        );
    });
}

#[test]
fn deactivate_identity_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register identity
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        // Deactivate
        assert_ok!(IdentityRegistry::deactivate_identity(RuntimeOrigin::signed(
            account
        )));

        // Check status
        let identity = IdentityRegistry::identities(account).unwrap();
        assert!(!identity.active);

        // Check event
        System::assert_last_event(Event::IdentityDeactivated { account }.into());
    });
}

#[test]
fn helper_is_active_identity_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Check not active before registration
        assert!(!IdentityRegistry::is_active_identity(&account));

        // Register
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        // Check active
        assert!(IdentityRegistry::is_active_identity(&account));

        // Deactivate
        assert_ok!(IdentityRegistry::deactivate_identity(RuntimeOrigin::signed(
            account
        )));

        // Check not active
        assert!(!IdentityRegistry::is_active_identity(&account));
    });
}

#[test]
fn helper_has_role_works() {
    new_test_ext().execute_with(|| {
        let account = 1u64;
        let did = create_did("patient1");
        let name = create_name("Alice");
        let email_hash = create_email_hash("alice@example.com");

        // Register as patient
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(account),
            did,
            UserRole::Patient,
            name,
            email_hash,
        ));

        // Check role
        assert!(IdentityRegistry::has_role(&account, UserRole::Patient));
        assert!(!IdentityRegistry::has_role(&account, UserRole::Researcher));
        assert!(!IdentityRegistry::has_role(&account, UserRole::Institution));
        assert!(!IdentityRegistry::has_role(&account, UserRole::Auditor));
    });
}

#[test]
fn helper_is_verified_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let auditor = 2u64;
        let patient_did = create_did("patient1");
        let auditor_did = create_did("auditor1");
        let name = create_name("User");
        let email_hash = create_email_hash("test@example.com");

        // Setup
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(patient),
            patient_did,
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));

        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(auditor),
            auditor_did,
            UserRole::Auditor,
            name,
            email_hash,
        ));

        // Check not verified
        assert!(!IdentityRegistry::is_verified(&patient));

        // Request and verify
        assert_ok!(IdentityRegistry::request_verification(RuntimeOrigin::signed(
            patient
        )));

        assert_ok!(IdentityRegistry::verify_identity(
            RuntimeOrigin::signed(auditor),
            patient,
        ));

        // Check verified
        assert!(IdentityRegistry::is_verified(&patient));
    });
}

#[test]
fn all_user_roles_work() {
    new_test_ext().execute_with(|| {
        let email_hash = create_email_hash("test@example.com");
        let name = create_name("Test User");

        // Test Patient role
        let patient = 1u64;
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(patient),
            create_did("patient1"),
            UserRole::Patient,
            name.clone(),
            email_hash,
        ));
        assert!(IdentityRegistry::has_role(&patient, UserRole::Patient));

        // Test Researcher role
        let researcher = 2u64;
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(researcher),
            create_did("researcher1"),
            UserRole::Researcher,
            name.clone(),
            email_hash,
        ));
        assert!(IdentityRegistry::has_role(&researcher, UserRole::Researcher));

        // Test Institution role
        let institution = 3u64;
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(institution),
            create_did("institution1"),
            UserRole::Institution,
            name.clone(),
            email_hash,
        ));
        assert!(IdentityRegistry::has_role(&institution, UserRole::Institution));

        // Test Auditor role
        let auditor = 4u64;
        assert_ok!(IdentityRegistry::register_identity(
            RuntimeOrigin::signed(auditor),
            create_did("auditor1"),
            UserRole::Auditor,
            name,
            email_hash,
        ));
        assert!(IdentityRegistry::has_role(&auditor, UserRole::Auditor));
    });
}
