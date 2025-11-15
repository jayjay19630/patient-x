//! Tests for consent-manager pallet

use crate::{mock::*, ConsentStatus, DataPurpose, DataType, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

#[test]
fn create_consent_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        // Register identities
        register_patient(patient);
        register_researcher(researcher);

        // Create consent
        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();
        let expires_at = 1000000u64;
        let terms_hash = H256::from_low_u64_be(1);

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types.clone(),
            expires_at,
            terms_hash,
        ));

        // Get consent ID from event
        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Verify consent was stored
        let consent = ConsentManager::consents(consent_id).unwrap();
        assert_eq!(consent.data_owner, patient);
        assert_eq!(consent.data_consumer, researcher);
        assert_eq!(consent.purpose, DataPurpose::Research);
        assert_eq!(consent.data_types, data_types);
        assert_eq!(consent.expires_at, expires_at);
        assert_eq!(consent.status, ConsentStatus::Active);
        assert_eq!(consent.terms_hash, terms_hash);
        assert_eq!(consent.access_count, 0);

        // Verify indices were updated
        assert!(ConsentManager::owner_consents(patient).contains(&consent_id));
        assert!(ConsentManager::consumer_consents(researcher).contains(&consent_id));
    });
}

#[test]
fn create_consent_fails_with_invalid_owner() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        // Only register researcher
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_noop!(
            ConsentManager::create_consent(
                RuntimeOrigin::signed(patient),
                researcher,
                DataPurpose::Research,
                data_types,
                1000000,
                H256::zero(),
            ),
            Error::<Test>::InvalidIdentity
        );
    });
}

#[test]
fn create_consent_fails_with_invalid_consumer() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let invalid = 2u64;

        // Only register patient
        register_patient(patient);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_noop!(
            ConsentManager::create_consent(
                RuntimeOrigin::signed(patient),
                invalid,
                DataPurpose::Research,
                data_types,
                1000000,
                H256::zero(),
            ),
            Error::<Test>::InvalidConsumer
        );
    });
}

#[test]
fn create_consent_fails_with_empty_data_types() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let empty_data_types = sp_runtime::BoundedVec::try_from(vec![]).unwrap();

        assert_noop!(
            ConsentManager::create_consent(
                RuntimeOrigin::signed(patient),
                researcher,
                DataPurpose::Research,
                empty_data_types,
                1000000,
                H256::zero(),
            ),
            Error::<Test>::InvalidDataTypes
        );
    });
}

#[test]
fn create_consent_fails_with_past_expiry() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        // Set current time
        Timestamp::set_timestamp(10000);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        // Try to create consent with past expiry
        assert_noop!(
            ConsentManager::create_consent(
                RuntimeOrigin::signed(patient),
                researcher,
                DataPurpose::Research,
                data_types,
                5000, // Past expiry
                H256::zero(),
            ),
            Error::<Test>::InvalidExpiryTime
        );
    });
}

#[test]
fn create_consent_works_with_institution() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let institution = 2u64;

        register_patient(patient);
        register_institution(institution);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            institution,
            DataPurpose::Treatment,
            data_types,
            1000000,
            H256::zero(),
        ));
    });
}

#[test]
fn revoke_consent_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        // Create consent
        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Revoke consent
        assert_ok!(ConsentManager::revoke_consent(
            RuntimeOrigin::signed(patient),
            consent_id,
        ));

        // Verify status
        let consent = ConsentManager::consents(consent_id).unwrap();
        assert_eq!(consent.status, ConsentStatus::Revoked);
        assert!(consent.revoked_at.is_some());

        // Check event
        System::assert_last_event(
            Event::ConsentRevoked {
                consent_id,
                revoker: patient,
            }
            .into(),
        );
    });
}

#[test]
fn revoke_consent_fails_when_not_owner() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Try to revoke as researcher
        assert_noop!(
            ConsentManager::revoke_consent(RuntimeOrigin::signed(researcher), consent_id),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn revoke_consent_fails_when_already_revoked() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Revoke first time
        assert_ok!(ConsentManager::revoke_consent(
            RuntimeOrigin::signed(patient),
            consent_id,
        ));

        // Try to revoke again
        assert_noop!(
            ConsentManager::revoke_consent(RuntimeOrigin::signed(patient), consent_id),
            Error::<Test>::AlreadyRevoked
        );
    });
}

#[test]
fn update_consent_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Update expiry
        let new_expiry = 2000000u64;
        assert_ok!(ConsentManager::update_consent(
            RuntimeOrigin::signed(patient),
            consent_id,
            Some(new_expiry),
            None,
        ));

        let consent = ConsentManager::consents(consent_id).unwrap();
        assert_eq!(consent.expires_at, new_expiry);

        // Update data types
        let new_data_types =
            sp_runtime::BoundedVec::try_from(vec![DataType::Imaging, DataType::Genomic]).unwrap();
        assert_ok!(ConsentManager::update_consent(
            RuntimeOrigin::signed(patient),
            consent_id,
            None,
            Some(new_data_types.clone()),
        ));

        let consent = ConsentManager::consents(consent_id).unwrap();
        assert_eq!(consent.data_types, new_data_types);
    });
}

#[test]
fn update_consent_fails_when_revoked() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Revoke consent
        assert_ok!(ConsentManager::revoke_consent(
            RuntimeOrigin::signed(patient),
            consent_id,
        ));

        // Try to update
        assert_noop!(
            ConsentManager::update_consent(
                RuntimeOrigin::signed(patient),
                consent_id,
                Some(2000000),
                None,
            ),
            Error::<Test>::ConsentExpired
        );
    });
}

#[test]
fn log_access_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Log access
        let data_hash = H256::from_low_u64_be(123);
        assert_ok!(ConsentManager::log_access(
            RuntimeOrigin::signed(researcher),
            consent_id,
            data_hash,
        ));

        // Verify access log
        let logs = ConsentManager::access_logs(consent_id);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].accessor, researcher);
        assert_eq!(logs[0].data_hash, data_hash);
        assert!(logs[0].approved);

        // Verify access count
        let consent = ConsentManager::consents(consent_id).unwrap();
        assert_eq!(consent.access_count, 1);
        assert!(consent.last_accessed.is_some());
    });
}

#[test]
fn log_access_fails_when_expired() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        // Create consent with near expiry
        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000, // Will expire soon
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Advance time past expiry
        Timestamp::set_timestamp(2000);

        // Try to log access
        assert_noop!(
            ConsentManager::log_access(
                RuntimeOrigin::signed(researcher),
                consent_id,
                H256::zero(),
            ),
            Error::<Test>::ConsentExpired
        );
    });
}

#[test]
fn check_consent_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Check consent
        assert_ok!(ConsentManager::check_consent(
            RuntimeOrigin::signed(researcher),
            consent_id,
            researcher,
        ));
    });
}

#[test]
fn check_consent_fails_with_wrong_accessor() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;
        let other = 3u64;

        register_patient(patient);
        register_researcher(researcher);
        register_researcher(other);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Try to check as different researcher
        assert_noop!(
            ConsentManager::check_consent(RuntimeOrigin::signed(other), consent_id, other),
            Error::<Test>::NotAuthorized
        );
    });
}

#[test]
fn all_data_purposes_work() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::All]).unwrap();
        let purposes = vec![
            DataPurpose::Research,
            DataPurpose::ClinicalTrial,
            DataPurpose::Treatment,
            DataPurpose::DrugDevelopment,
            DataPurpose::PublicHealth,
            DataPurpose::MachineLearning,
            DataPurpose::Other,
        ];

        for purpose in purposes {
            assert_ok!(ConsentManager::create_consent(
                RuntimeOrigin::signed(patient),
                researcher,
                purpose,
                data_types.clone(),
                1000000,
                H256::zero(),
            ));
        }
    });
}

#[test]
fn all_data_types_work() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![
            DataType::All,
            DataType::LabResults,
            DataType::Imaging,
            DataType::Prescriptions,
            DataType::Diagnosis,
            DataType::Genomic,
            DataType::Vitals,
            DataType::Demographics,
        ])
        .unwrap();

        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            1000000,
            H256::zero(),
        ));
    });
}

#[test]
fn helper_get_active_consents_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        // Create multiple consents
        for _ in 0..3 {
            assert_ok!(ConsentManager::create_consent(
                RuntimeOrigin::signed(patient),
                researcher,
                DataPurpose::Research,
                data_types.clone(),
                1000000,
                H256::zero(),
            ));
        }

        // Get active consents for owner
        let owner_consents = ConsentManager::get_active_consents_for_owner(&patient);
        assert_eq!(owner_consents.len(), 3);

        // Get active consents for consumer
        let consumer_consents = ConsentManager::get_active_consents_for_consumer(&researcher);
        assert_eq!(consumer_consents.len(), 3);

        // Revoke one consent
        let consent_id = ConsentManager::owner_consents(patient)[0];
        assert_ok!(ConsentManager::revoke_consent(
            RuntimeOrigin::signed(patient),
            consent_id,
        ));

        // Check counts decreased
        let owner_consents = ConsentManager::get_active_consents_for_owner(&patient);
        assert_eq!(owner_consents.len(), 2);
    });
}

#[test]
fn consent_with_no_expiry_works() {
    new_test_ext().execute_with(|| {
        let patient = 1u64;
        let researcher = 2u64;

        register_patient(patient);
        register_researcher(researcher);

        let data_types = sp_runtime::BoundedVec::try_from(vec![DataType::LabResults]).unwrap();

        // Create consent with no expiry (0)
        assert_ok!(ConsentManager::create_consent(
            RuntimeOrigin::signed(patient),
            researcher,
            DataPurpose::Research,
            data_types,
            0, // No expiry
            H256::zero(),
        ));

        let events = System::events();
        let consent_id = if let RuntimeEvent::ConsentManager(Event::ConsentCreated {
            consent_id,
            ..
        }) = events.last().unwrap().event
        {
            consent_id
        } else {
            panic!("Expected ConsentCreated event");
        };

        // Advance time significantly
        Timestamp::set_timestamp(999999999);

        // Should still be valid
        assert_ok!(ConsentManager::check_consent(
            RuntimeOrigin::signed(researcher),
            consent_id,
            researcher,
        ));
    });
}
